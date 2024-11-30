use crate::plugin::PluginImpl;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{ffi::c_void, num::NonZero, ptr::NonNull, sync::Arc};
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};

pub struct PluginUiImpl {
    notification_receiver: UnboundedReceiver<UiNotification>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "payload")]
pub enum UiNotification {
    UpdatePlayingState(bool),
}

impl PluginUiImpl {
    pub unsafe fn new(
        handle: usize,
        plugin: Arc<Mutex<PluginImpl>>,
        width: usize,
        height: usize,
        scale_factor: f64,
    ) -> Result<Self> {
        let raw_window_handle = if cfg!(target_os = "windows") {
            raw_window_handle::RawWindowHandle::Win32(raw_window_handle::Win32WindowHandle::new(
                NonZero::new(handle as isize).ok_or_else(|| anyhow::anyhow!("handle is zero"))?,
            ))
        } else if cfg!(target_os = "macos") {
            raw_window_handle::RawWindowHandle::AppKit(raw_window_handle::AppKitWindowHandle::new(
                NonNull::new(handle as *mut c_void)
                    .ok_or_else(|| anyhow::anyhow!("handle is zero"))?
                    .cast(),
            ))
        } else if cfg!(target_os = "linux") {
            raw_window_handle::RawWindowHandle::Xcb(raw_window_handle::XcbWindowHandle::new(
                NonZero::new(handle as u32).ok_or_else(|| anyhow::anyhow!("handle is zero"))?,
            ))
        } else {
            unreachable!()
        };
        let window_handle = raw_window_handle::WindowHandle::borrow_raw(raw_window_handle);

        let (notification_sender, notification_receiver) = tokio::sync::mpsc::unbounded_channel();
        {
            let mut plugin = plugin.blocking_lock();
            plugin.notification_sender = Some(notification_sender);
        }

        let plugin_ref = Arc::clone(&plugin);

        Ok(PluginUiImpl {
            notification_receiver,
        })
    }

    pub fn idle(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn set_size(&self, width: usize, height: usize, scale_factor: f64) -> Result<()> {
        Ok(())
    }
}
