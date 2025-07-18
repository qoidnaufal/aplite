use std::collections::HashMap;
use std::sync::Arc;

use aplite_reactive::{Effect, Update, With};
use winit::dpi::{PhysicalPosition, PhysicalSize, LogicalSize};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::application::ApplicationHandler;

use aplite_types::{Size, Rgba};
use aplite_renderer::{Renderer, RendererError};

use crate::prelude::ApliteResult;
use crate::context::Context;
use crate::error::ApliteError;
use crate::view::{IntoView, View, ViewId, VIEW_STORAGE};

pub(crate) const DEFAULT_SCREEN_SIZE: Size<u32> = Size::new(800, 600);

pub struct WindowAttributes {
    title: &'static str,
    inner_size: Size<u32>,
    decorations: bool,
    transparent: bool,
    maximized: bool,
    resizable: bool,
}

impl Default for WindowAttributes {
    fn default() -> Self {
        Self {
            title: "GUI App",
            inner_size: DEFAULT_SCREEN_SIZE,
            decorations: true,
            transparent: false,
            maximized: false,
            resizable: true,
        }
    }
}

#[cfg(target_os = "macos")]
impl From<&WindowAttributes> for winit::window::WindowAttributes {
    fn from(w: &WindowAttributes) -> Self {
        use winit::platform::macos::WindowAttributesExtMacOS;

        Self::default()
            .with_inner_size(LogicalSize::new(w.inner_size.width(), w.inner_size.height()))
            .with_title(w.title)
            .with_transparent(w.transparent)
            .with_maximized(w.maximized)
            .with_resizable(w.resizable)
            .with_titlebar_hidden(!w.decorations)
    }
}

#[cfg(not(target_os = "macos"))]
impl From<&WindowAttributes> for winit::window::WindowAttributes {
    fn from(w: &WindowAttributes) -> Self {
        Self::default()
            .with_inner_size(LogicalSize::new(w.inner_size.width(), w.inner_size.height()))
            .with_title(w.title)
            .with_transparent(w.transparent)
            .with_maximized(w.maximized)
            .with_resizable(w.resizable)
            .with_decorations(w.decorations)
    }
}

pub struct Aplite {
    renderer: Option<Renderer>,
    cx: Context,
    window: HashMap<WindowId, Arc<Window>>,
    root_view_id: HashMap<WindowId, ViewId>,
    window_attributes: WindowAttributes,
    pending_views: Vec<Box<dyn FnOnce(WindowId) -> Box<dyn IntoView>>>,

    #[cfg(feature = "render_stats")]
    stats: aplite_stats::Stats,
}

// user API
impl Aplite {
    pub fn new<IV: IntoView + 'static>(view_fn: impl FnOnce() -> IV + 'static) -> Self {
        let mut app = Self::new_empty();
        app.pending_views.push(Box::new(|_| Box::new(view_fn())));
        app
    }

    pub fn new_empty() -> Self {
        Self {
            renderer: None,
            cx: Context::new(),
            window: HashMap::with_capacity(4),
            root_view_id: HashMap::with_capacity(4),
            window_attributes: WindowAttributes::default(),
            pending_views: Vec::with_capacity(4),

            #[cfg(feature = "render_stats")]
            stats: aplite_stats::Stats::new(),
        }
    }

    pub fn launch(mut self) -> ApliteResult {
        let event_loop = EventLoop::new()?;
        event_loop.run_app(&mut self)?;

        Ok(())
    }

    pub fn with_title(mut self, title: &'static str) -> Self {
        self.window_attributes.title = title;
        self
    }

    pub fn with_inner_size(mut self, width: u32, height: u32) -> Self {
        self.window_attributes.inner_size = (width, height).into();
        self
    }

    pub fn with_decorations_enabled(mut self, val: bool) -> Self {
        self.window_attributes.decorations = val;
        self
    }

    pub fn with_transparent(mut self, val: bool) -> Self {
        self.window_attributes.transparent = val;
        self
    }

    pub fn with_maximized(mut self, val: bool) -> Self {
        self.window_attributes.maximized = val;
        self
    }

    pub fn with_resizable(mut self, val: bool) -> Self {
        self.window_attributes.resizable = val;
        self
    }

    pub fn with_background_color(self, color: Rgba<u8>) -> Self {
        let _ = color;
        self
    }
}

// initialization
impl Aplite {
    fn initialize_window(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(ViewId, Arc<Window>), ApliteError> {
        let attributes = &self.window_attributes;
        let window = Self::create_window(event_loop, attributes)?;
        let window_id = window.id();

        let view_id = VIEW_STORAGE.with(|s| {
            let size = window
                .inner_size()
                .to_logical(window.scale_factor());

            let root = s.create_entity();
            let root_view = View::window(Size::new(size.width, size.height));

            s.storage.borrow_mut().insert(root, root_view);
            self.root_view_id.insert(window_id, root);
            self.window.insert(window_id, Arc::clone(&window));

            if let Some(view_fn) = self.pending_views.pop() {
                let view = view_fn(window_id);
                view.widget_state().z_index.update(|z_index| *z_index += 1);
                s.append_child(&root, view);

                self.cx.layout_the_whole_window(&root);

                #[cfg(feature = "debug_tree")] eprintln!("{:?}", s.tree.borrow());
            }

            root
        });

        Ok((view_id, window))
    }

    /// Create new [`Window`]
    fn create_window(
        event_loop: &ActiveEventLoop,
        attributes: &WindowAttributes,
    ) -> Result<Arc<Window>, ApliteError> {
        let window = event_loop.create_window(attributes.into())?;
        Ok(Arc::new(window))
    }

    /// Initialize the [`Renderer`]
    fn initialize_renderer(&mut self, window: Arc<Window>) -> Result<(), ApliteError> {
        let renderer = Renderer::new(Arc::clone(&window))?;
        self.renderer = Some(renderer);
        Ok(())
    }

    /// Track the [`Window`] with the associated root [`ViewId`] for rendering
    fn track_window(&mut self, view_id: ViewId, window: Arc<Window>) {
        let dirty = self.cx.dirty();

        Effect::new(move |_| {
            dirty.with(|root_id| {
                if root_id.is_some_and(|id| id == view_id) {
                    window.request_redraw();
                }
            })
        });
    }
}

// window event
impl Aplite {
    fn handle_resize(&mut self, size: PhysicalSize<u32>, window_id: WindowId) {
        if let Some(renderer) = self.renderer.as_mut() {
            let logical = size.to_logical::<u32>(renderer.scale_factor());
            let root_id = self.root_view_id[&window_id];
            VIEW_STORAGE.with(|s| {
                if let Some(window_state) = s.storage.borrow().get(&root_id) {
                    window_state.widget_state().rect.update_untracked(|rect| {
                        rect.set_size(Size::new(logical.width, logical.height));
                    });
                }
            });
            crate::context::layout::LayoutContext::new(root_id).calculate();
            renderer.resize(Size::new(logical.width, logical.height));
        }
    }

    fn set_scale_factor(&mut self, scale_factor: f64) {
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.set_scale_factor(scale_factor);
        }
    }

    fn handle_mouse_move(&mut self, window_id: &WindowId, pos: PhysicalPosition<f64>) {
        if let Some(renderer) = self.renderer.as_mut()
            && let Some(root) = self.root_view_id.get(window_id) {
            let logical_pos = pos.to_logical::<f32>(renderer.scale_factor());
            self.cx.handle_mouse_move(root, (logical_pos.x, logical_pos.y));
        }
    }

    fn handle_click(&mut self, state: ElementState, button: MouseButton) {
        self.cx.handle_click(state, button);
    }

    fn handle_close_request(&mut self, window_id: &WindowId, event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.remove(window_id) {
            drop(window);
            event_loop.exit();
        }
    }

    // WARN: not sure if retained mode works like this
    fn handle_redraw_request(&mut self, window_id: &WindowId, event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.get(window_id).cloned()
        && let Some(root_id) = self.root_view_id.get(window_id)
        && let Some(renderer) = self.renderer.as_mut()
        {
            #[cfg(feature = "render_stats")] let start = std::time::Instant::now();

            renderer.begin();
            self.cx.prepare_data(*root_id, renderer);
            // TODO: this should be window.pre_present_notify(),
            // and the renderer.finish()
            if let Err(err) = renderer.render(Rgba::TRANSPARENT, window) {
                let size = renderer.screen_res().u32();
                match err {
                    RendererError::ShouldResize => renderer.resize(size),
                    RendererError::ShouldExit => event_loop.exit(),
                    _ => {}
                }
            }

            #[cfg(feature = "render_stats")] self.stats.inc(start.elapsed());
        }
    }
}

impl ApplicationHandler for Aplite {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self.initialize_window(event_loop) {
            Ok((view_id, window)) if self
                .initialize_renderer(Arc::clone(&window))
                .is_ok() => self.track_window(view_id, window),
            _ => event_loop.exit(),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => self.handle_close_request(&window_id, event_loop),
            WindowEvent::RedrawRequested => self.handle_redraw_request(&window_id, event_loop),
            WindowEvent::Resized(s) => self.handle_resize(s, window_id),
            WindowEvent::MouseInput { state, button, .. } => self.handle_click(state, button),
            WindowEvent::CursorMoved { position, .. } => self.handle_mouse_move(&window_id, position),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => self.set_scale_factor(scale_factor),
            _ => {}
        }
    }
}

