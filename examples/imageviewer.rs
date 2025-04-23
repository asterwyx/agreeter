use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_output, delegate_registry, delegate_shm, delegate_xdg_shell,
    delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        WaylandSurface,
        xdg::{
            XdgShell,
            window::{Window, WindowConfigure, WindowDecorations, WindowHandler},
        },
    },
    shm::{
        Shm, ShmHandler,
        slot::{Buffer, SlotPool},
    },
};
use wayland_client::{
    Connection, QueueHandle,
    globals::registry_queue_init,
    protocol::{wl_output, wl_shm, wl_surface},
};

fn main() {
    env_logger::init();

    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();
    let mut state = State {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        compositor_state: CompositorState::bind(&globals, &qh)
            .expect("wl_compositor not available"),
        shm_state: Shm::bind(&globals, &qh).expect("wl_shm not available"),
        xdg_shell_state: XdgShell::bind(&globals, &qh).expect("xdg shell not available"),
        pool: None,
        windows: Vec::new(),
    };
    let image = match image::open("/home/astrea/Pictures/Avatars/Caption.jpg") {
        Ok(image) => image,
        Err(e) => {
            println!("Failed to load image: {}", e);
            return;
        }
    };
    let image = image.to_rgba8();

    let surface = state.compositor_state.create_surface(&qh);
    let pool_size = image.width() * image.height() * 4;
    let window =
        state
            .xdg_shell_state
            .create_window(surface, WindowDecorations::ServerDefault, &qh);
    window.set_title("Image Viewer");
    window.set_app_id("agreeter");
    window.commit();

    state.windows.push(ImageViewer {
        width: image.width(),
        height: image.height(),
        window,
        image,
        first_configure: true,
        damaged: true,
        buffer: None,
    });
    let pool = SlotPool::new(pool_size as usize, &state.shm_state).expect("Failed to create pool");
    state.pool = Some(pool);
    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();
        if state.windows.is_empty() {
            println!("exiting example");
            break;
        }
    }
}

struct ImageViewer {
    window: Window,
    image: image::RgbaImage,
    width: u32,
    height: u32,
    buffer: Option<Buffer>,
    first_configure: bool,
    damaged: bool,
}

struct State {
    registry_state: RegistryState,
    output_state: OutputState,
    compositor_state: CompositorState,
    shm_state: Shm,
    xdg_shell_state: XdgShell,

    pool: Option<SlotPool>,
    windows: Vec<ImageViewer>,
}

impl CompositorHandler for State {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        println!("Frame event: {:?}", _time);
        self.draw(_conn, _qh);
    }
    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for State {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        println!("New output added: {:?}", _output);
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        println!("Output updated: {:?}", _output);
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        println!("Output destroyed: {:?}", _output);
    }
}

impl ShmHandler for State {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm_state
    }
}

impl WindowHandler for State {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, window: &Window) {
        self.windows.retain(|viewer| viewer.window != *window);
    }
    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        window: &Window,
        configure: WindowConfigure,
        _serial: u32,
    ) {
        for viewer in &mut self.windows {
            if viewer.window == *window {
                continue;
            }
            viewer.buffer = None;
            viewer.width = configure.new_size.0.map(|v| v.get()).unwrap_or(256);
            viewer.height = configure.new_size.1.map(|v| v.get()).unwrap_or(256);
            viewer.damaged = true;

            viewer.first_configure = false;
        }
    }
}

impl State {
    pub fn draw(&mut self, _conn: &Connection, qh: &QueueHandle<Self>) {
        for viewer in &mut self.windows {
            if viewer.first_configure || !viewer.damaged {
                continue;
            }
            let window = &viewer.window;
            let width = viewer.width;
            let height = viewer.height;
            let stride = viewer.width as i32 * 4;
            let pool = self.pool.as_mut().unwrap();
            let buffer = viewer.buffer.get_or_insert_with(|| {
                pool.create_buffer(
                    width as i32,
                    height as i32,
                    stride,
                    wl_shm::Format::Argb8888,
                )
                .expect("create buffer")
                .0
            });
            let canvas = match pool.canvas(buffer) {
                Some(canvas) => canvas,
                None => {
                    let (second_buffer, canvas) = pool
                        .create_buffer(
                            viewer.width as i32,
                            viewer.height as i32,
                            stride,
                            wl_shm::Format::Argb8888,
                        )
                        .expect("create buffer");
                    *buffer = second_buffer;
                    canvas
                }
            };
            {
                let image = image::imageops::resize(
                    &viewer.image,
                    viewer.width,
                    viewer.height,
                    image::imageops::FilterType::Nearest,
                );
                for (pixel, argb) in image.pixels().zip(canvas.chunks_exact_mut(4)) {
                    argb[3] = pixel.0[3];
                    argb[2] = pixel.0[0];
                    argb[1] = pixel.0[1];
                    argb[0] = pixel.0[2];
                }
            }
            window
                .wl_surface()
                .damage_buffer(0, 0, viewer.width as i32, viewer.height as i32);
            viewer.damaged = false;

            window.wl_surface().frame(qh, window.wl_surface().clone());

            buffer
                .attach_to(window.wl_surface())
                .expect("buffer attach");
            window.wl_surface().commit();
        }
    }
}

delegate_compositor!(State);
delegate_output!(State);
delegate_shm!(State);

delegate_xdg_shell!(State);
delegate_xdg_window!(State);

delegate_registry!(State);

impl ProvidesRegistryState for State {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers!(OutputState);
}
