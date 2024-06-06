use std::fmt::{self, Formatter, Debug};

use libcamera::{camera::CameraConfiguration, control::ControlList, framebuffer_allocator::FrameBufferAllocator, stream::{Stream, StreamConfigurationRef, StreamRole}};

pub struct RpiCamera {
    camera_manager: libcamera::camera_manager::CameraManager,
    picked_camera: usize,
}

impl RpiCamera {
    pub fn new() -> Result<Self, Error> {
        let camera_manager = libcamera::camera_manager::CameraManager::new()?;

        // pick the first camera we found
        {
            let cameras = camera_manager.cameras();
            if cameras.is_empty() {
                return Err(Error::NoCameraAvailable);
            }
        }

        Ok(RpiCamera {
            camera_manager,
            picked_camera: 0,
        })
    }

    #[tracing::instrument]
    pub fn capture(&self) -> Result<(), Error> {
        tracing::debug!("finding and acquiring camera");
        let camera = self.camera_manager.cameras()
        .get(self.picked_camera)
        .ok_or(Error::CameraUnavailable(self.picked_camera))?;

        let active_camera = camera.acquire()?;
        tracing::debug!(properties = ?active_camera.properties(), "camera properties");

        tracing::info!("starting camera");
        active_camera.start(None)?;

        let camera_configuration = active_camera.generate_configuration(&[
            StreamRole::StillCapture,
        ]).ok_or(Error::ConfigurationCreationFailed)?;
        active_camera.configure(&mut camera_configuration)?;

        let mut frame_buffer_allocator = FrameBufferAllocator::new(&active_camera);
        frame_buffer_allocator.alloc(stream)



        let capture_request = active_camera.create_request(Some(1))
            .ok_or(Error::RequestCreationFailed)?;
        capture_request.add_buffer(stream, buffer)
        active_camera.queue_request(capture_request);


        let mut stream = camera.create_stream()?;
        stream.start()?;
        let frame = stream.get_frame()?;
        stream.stop()?;
        Ok(frame)
    }
}

impl Debug for RpiCamera {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("RpiCamera")
            .field("picked_camera", &self.picked_camera)
            .finish()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("i/o error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("no camera available")]
    NoCameraAvailable,

    #[error("camera {0} unavailable")]
    CameraUnavailable(usize),

    #[error("unable to create camera configuration")]
    ConfigurationCreationFailed,
    #[error("unable to create request")]
    RequestCreationFailed,
}
