//! Message queue.

use std::fmt::{self, Formatter, Debug};

use opentelemetry::global::get_text_map_propagator;
use paho_mqtt::async_client::AsyncClient as MqttClient;
use paho_mqtt::{MessageBuilder as MqttMessageBuilder, Properties as MqttProps, Property, PropertyCode};
use prost::Message as ProstMessage;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use crate::eventpb;

/// The message queue that sends the event to the Central system.
pub struct CentralMessageQueue {
    client: MqttClient,
    device_id: String,
}

impl CentralMessageQueue {
    pub fn new<T: AsRef<str>>(server_uri: &str, device_id: T) -> Result<CentralMessageQueue, Error> {
        let client = MqttClient::new(
            paho_mqtt::CreateOptionsBuilder::new()
                .server_uri(server_uri)
                .client_id(device_id.as_ref())
                .finalize(),
        )?;

        Ok(CentralMessageQueue {
            client,
            device_id: device_id.as_ref().to_string(),
        })
    }

    /// Create the message queue from the environment variable.
    pub fn new_from_env() -> Result<CentralMessageQueue, Error> {
        let server_uri = std::env::var("IOT_EDGE_MQTT_SERVER_URI").expect("MQTT_SERVER_URI is not set");
        let device_id = std::env::var("IOT_EDGE_DEVICE_ID").expect("DEVICE_ID is not set");

        CentralMessageQueue::new(&server_uri, device_id)
    }

    /// Connect to the MQTT broker.
    ///
    /// You must call this method before calling `publish`.
    #[tracing::instrument(err)]
    pub async fn connect(&self) -> Result<(), Error> {
        tracing::info!("connect to the MQTT broker");

        let connection_info = self.client.connect(None).await;
        if let Err(e) = connection_info {
            tracing::error!(error = ?e, "failed to connect to the MQTT broker");
            return Err(e.into());
        }

        tracing::info!(info = ?connection_info, "connected to the MQTT broker");
        Ok(())
    }

    /// Publish the event to the specified topic.
    #[tracing::instrument(err)]
    pub async fn publish(&self, event: eventpb::EventMessage) -> Result<(), Error> {
        tracing::info!("packaging the event to MQTT Message");
        let Some(event_type) = event.get_event_type() else {
            tracing::warn!("no event are given");
            return Err(Error::NoEventGiven);
        };
        let topic = format!("iot/events/v1/{}", event_type);

        let event_id = uuid::Uuid::now_v7();
        let event_id_str = event_id.to_string();

        let event_emitted_at = chrono::Utc::now()
            .to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);

        let marshalled_event = event.encode_to_vec();

        tracing::info!(event_id_str, event_emitted_at, "putting properties");

        let mut message_properties = MqttProps::new();
        message_properties.push(Property::new_string(PropertyCode::ContentType, "application/x-google-protobuf")?)?;
        message_properties.push(Property::new_string_pair(PropertyCode::UserProperty, "event_id", &event_id_str)?)?;
        message_properties.push(Property::new_string_pair(PropertyCode::UserProperty, "device_id", &self.device_id)?)?;
        message_properties.push(Property::new_string_pair(PropertyCode::UserProperty, "emitted_at", &event_emitted_at)?)?;

        // tracing information
        let ctx = tracing::Span::current().context();
        get_text_map_propagator(|propagator| {
            propagator.inject_context(&ctx, &mut MqttCarrierInjector(&mut message_properties))
        });

        let message = MqttMessageBuilder::new()
            .topic(topic)
            .payload(marshalled_event)
            .qos(0)
            .properties(message_properties)
            .finalize();

        tracing::info!(?message, "publishing the event to the MQTT broker");
        self.client.publish(message).await?;

        tracing::info!("event published successfully");
        Ok(())
    }
}

impl Debug for CentralMessageQueue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("CentralMessageQueue")
            .field("device_id", &self.device_id)
            .finish()
    }
}

pub struct MqttCarrierInjector<'a>(pub &'a mut MqttProps);

impl<'a> opentelemetry::propagation::Injector for MqttCarrierInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        self.0.push_string_pair(PropertyCode::UserProperty, key, &value)
            .expect("cannot push string pair")
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("create MQTT client: {0}")]
    CreateMqttClient(#[from] paho_mqtt::Error),

    #[error("no event are given")]
    NoEventGiven,

    #[error("encode event to bytes: {0}")]
    EncodeEvent(#[from] prost::EncodeError),
}
