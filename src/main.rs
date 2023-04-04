use pixels::{Error, Pixels, SurfaceTexture};
use rand::prelude::*;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> Result<(), Error> {
    println!("Random start: {}", random::<i32>());
    let width = 400;
    let height = 400;
    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(width as f64, height as f64);
        WindowBuilder::new()
            .with_title("Crabs")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(width, height, surface_texture)?
    };

    let mut world = World::new([54, 139, 187, 255], [190, 25, 49, 255], width, height);

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame_mut());
            if pixels
                .render()
                .map_err(|e| panic!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
            }
        }
        if let Event::WindowEvent { event: e, .. } = event {
            if e == WindowEvent::CloseRequested {
                *control_flow = ControlFlow::Exit
            } else if let WindowEvent::Resized(new_size) = e {
                println!("{:?}", new_size);
                pixels
                    .resize_surface(new_size.width, new_size.height)
                    .unwrap();
            }
        }

        window.request_redraw();
    });
}

pub struct World {
    clear_color: [u8; 4],
    width: u32,
    height: u32,
    crabs: Vec<bool>,
    crab_buffer: Vec<bool>,
    crab_color: [u8; 4],
}

impl World {
    pub fn new(clear_color: [u8; 4], crab_color: [u8; 4], width: u32, height: u32) -> Self {
        let w = width as usize;
        let h = height as usize;
        let mut crabs = vec![false; w * h];
        crabs.iter_mut().for_each(|c| *c = random());
        let crab_buffer = vec![false; w * h];

        Self {
            clear_color,
            crab_color,
            width,
            height,
            crabs,
            crab_buffer,
        }
    }
    pub fn get_crab(&self, x: usize, y: usize) -> bool {
        self.crabs[x + (self.height as usize * y)]
    }
    pub fn set_crab(&mut self, val: bool, x: usize, y: usize) {
        self.crabs[x + (self.height as usize * y)] = val;
    }
    /// 0 1 2 \
    /// 3 _ 4 \
    /// 5 6 7 \
    pub fn get_crabs_siblings(&self, x: usize, y: usize) -> impl Iterator<Item = bool> {
        // TODO add support for corners and edges
        //println!("{}", x + (self.height as usize * y));
        if x == 0 || y == 0 || y >= self.height as usize - 1 || x >= self.width as usize - 1 {
            vec![].into_iter()
        } else {
            let c = &self.crabs;
            #[allow(clippy::identity_op)]
            vec![
                c[(x - 1) + (self.height as usize * (y - 1))], // 0 (-1, -1)
                c[(x + 0) + (self.height as usize * (y - 1))], // 1 ( 0, -1)
                c[(x + 1) + (self.height as usize * (y - 1))], // 2 ( 1, -1)
                c[(x - 1) + (self.height as usize * (y + 0))], // 3 (-1,  0)
                c[(x + 1) + (self.height as usize * (y + 0))], // 4 ( 1   0)
                c[(x - 1) + (self.height as usize * (y + 1))], // 5 (-1,  1)
                c[(x + 0) + (self.height as usize * (y + 1))], // 6 ( 0,  1)
                c[(x + 1) + (self.height as usize * (y + 1))], // 7 ( 1,  1)
            ]
            .into_iter()
        }
    }

    pub fn should_crab_live(&self, x: usize, y: usize) -> bool {
        // 1. Any live cell with two or three live neighbours survives.
        // 2. Any dead cell with three live neighbours becomes a live cell.
        // 3. All other live cells die in the next generation. Similarly, all other dead cells stay dead.
        let alive = self.get_crabs_siblings(x, y).filter(|c| *c).count();
        if self.get_crab(x, y) {
            alive == 2 || alive == 3
        } else {
            alive == 3
        }
    }

    pub fn draw(&mut self, frame: &mut [u8]) {
        for i in 0..self.crabs.len() {
            let x = (i % self.width as usize) as usize;
            let y = (i / self.width as usize) as usize;
            let should_live = self.should_crab_live(x, y);
            self.crab_buffer[i] = should_live;

            let pixel = frame.chunks_exact_mut(4).nth(i).unwrap();
            if should_live {
                pixel.copy_from_slice(&self.crab_color);
            } else {
                pixel.copy_from_slice(&self.clear_color);
            }
        }
        self.crabs.copy_from_slice(&self.crab_buffer);
    }
}
