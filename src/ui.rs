use crate::plugin::PluginImpl;
use anyhow::Result;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use serde::{Deserialize, Serialize};
use std::{ffi::c_void, sync::Arc};
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};

#[derive(Debug, Clone)]
struct State {
    // Your state here
}

impl State {
    fn new() -> Self {
        Self {}
    }
}

pub struct PluginUiImpl {
    notification_receiver: UnboundedReceiver<UiNotification>,

    window_handle: ParentWindow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "payload")]
pub enum UiNotification {
    UpdatePlayingState(bool),
}

pub struct ParentWindow(pub *mut ::std::ffi::c_void);

#[cfg(target_os = "macos")]
unsafe impl HasRawWindowHandle for ParentWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::AppKitWindowHandle;

        let mut handle = AppKitWindowHandle::empty();
        handle.ns_view = self.0;

        RawWindowHandle::AppKit(handle)
    }
}

#[cfg(target_os = "windows")]
unsafe impl HasRawWindowHandle for ParentWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::Win32WindowHandle;

        let mut handle = Win32WindowHandle::empty();
        handle.hwnd = self.0;

        RawWindowHandle::Win32(handle)
    }
}

#[cfg(target_os = "linux")]
unsafe impl HasRawWindowHandle for ParentWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::XcbWindowHandle;

        let mut handle = XcbWindowHandle::empty();
        handle.window = self.0 as u32;

        RawWindowHandle::Xcb(handle)
    }
}

impl PluginUiImpl {
    pub unsafe fn new(
        raw_handle: usize,
        plugin: Arc<Mutex<PluginImpl>>,
        width: usize,
        height: usize,
        scale_factor: f64,
    ) -> Result<Self> {
        let (notification_sender, notification_receiver) = tokio::sync::mpsc::unbounded_channel();
        {
            let mut plugin = plugin.blocking_lock();
            plugin.notification_sender = Some(notification_sender);
        }

        let settings = baseview::WindowOpenOptions {
            title: String::from("egui-baseview hello world"),
            size: baseview::Size::new(width as f64, height as f64),
            scale: baseview::WindowScalePolicy::SystemScaleFactor,
            gl_config: Some(Default::default()),
        };

        let state = ();
        let window_handle = ParentWindow(raw_handle as *mut c_void);
        let window = egui_baseview::EguiWindow::open_parented(
            &window_handle,
            settings,
            egui_baseview::GraphicsConfig::default(),
            state,
            |_egui_ctx: &egui::Context, _queue: &mut egui_baseview::Queue, _state: &mut ()| {},
            |egui_ctx: &egui::Context, _queue: &mut egui_baseview::Queue, _state: &mut ()| {
                egui::Window::new("egui-baseview hello world").show(egui_ctx, |ui| {
                    ui.label("Hello World!");
                });
            },
        );

        Ok(PluginUiImpl {
            notification_receiver,

            window_handle,
        })
    }

    pub fn idle(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn set_size(&self, width: usize, height: usize, scale_factor: f64) -> Result<()> {
        Ok(())
    }
}
