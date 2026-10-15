use bootloader_api::info::{FrameBuffer, FrameBufferInfo, Optional};
use bootloader_x86_64_common::logger::LockedLogger;
use conquer_once::spin::OnceCell;

pub static LOGGER: OnceCell<LockedLogger> = OnceCell::uninit();

fn init_logger(buffer: &'static mut [u8], info: FrameBufferInfo) {
    let logger = LOGGER.get_or_init(move || LockedLogger::new(buffer, info, false, true));
    log::set_logger(logger).expect("Logger already set");
    log::set_max_level(log::LevelFilter::Info);
}

pub(crate) fn init(frame_buffer_optional: &'static mut Optional<FrameBuffer>) {
    let frame_buffer_option = frame_buffer_optional.as_mut();
    let frame_buffer_struct = frame_buffer_option.unwrap();
    let frame_buffer_info = frame_buffer_struct.info();
    let raw_frame_buffer = frame_buffer_struct.buffer_mut();
    init_logger(raw_frame_buffer, frame_buffer_info);
    log::info!("Logger initialized");
}
