use kiss3d::window::Window;
use kiss3d::scene::{SceneNode, PlanarSceneNode};
use kiss3d::nalgebra::{Translation2};
use kiss3d::event::{WindowEvent, Action, MouseButton, Key};

use std::time::{Duration, Instant};
use std::collections::HashMap;

use rand;

const WINDOW_WIDTH: u32 = 1000;
const WINDOW_HEIGHT: u32 = 750;
const UNIT_SIZE: f32 = 10.0;
const GRID_WIDTH: usize = (WINDOW_WIDTH as f32 / UNIT_SIZE) as usize;
const GRID_HEIGHT: usize = (WINDOW_HEIGHT as f32 / UNIT_SIZE) as usize;

enum AppState {
    UserInput, 
    Simulation, 
}

fn key_cooldown(key: &Key) -> Duration {
    match *key {
        Key::R => Duration::from_millis(500), 
        Key::C => Duration::from_millis(500),
        _ => Duration::from_millis(250), 
    }
}

fn main() {
    let mut tick_duration = Duration::from_millis(500);

    let mut app_state = AppState::UserInput;

    let mut window = Window::new_with_size("Conway", WINDOW_WIDTH, WINDOW_HEIGHT);  
    window.set_framerate_limit(Some(500));

    let mut grid = Grid::new();

    let mut events = window.events();

    grid.data[30][11] = true;
    grid.data[30][12] = true;
    grid.data[30][13] = true;
    grid.data[31][13] = true;   
    grid.data[32][12] = true;

    grid.draw(&mut window);

    let mut input_timer = Instant::now();
    let input_cooldown = Duration::from_millis(250);

    let mut mouse_is_pressed = false;
    let mut mode = true;

    let mut mouse_cell_x_index = 0;
    let mut mouse_cell_y_index = 0;

    let mut timer = Instant::now();

    while window.render() {
        let loop_timer = Instant::now();

        match app_state {
            AppState::UserInput => {
                for event in events.iter() {
                    match event.value {
                        WindowEvent::CursorPos(y, x, _) => {
                            mouse_cell_x_index = (x / UNIT_SIZE as f64 / 2.0) as usize;
                            mouse_cell_y_index = (y / UNIT_SIZE as f64 / 2.0) as usize;

                            if mouse_cell_x_index >= GRID_HEIGHT {
                                mouse_cell_x_index = GRID_HEIGHT - 1;
                            }

                            if mouse_cell_y_index >= GRID_WIDTH {
                                mouse_cell_y_index = GRID_WIDTH - 1;
                            }

                            if mouse_is_pressed {
                                grid.data[mouse_cell_x_index][mouse_cell_y_index] = mode;

                                grid.draw(&mut window);
                            }
                        }

                        WindowEvent::MouseButton(mouse_button, action, _) => {
                            if action == Action::Press {
                                mouse_is_pressed = true;

                                mode = !grid.data[mouse_cell_x_index][mouse_cell_y_index];
                                grid.data[mouse_cell_x_index][mouse_cell_y_index] = mode;

                                grid.draw(&mut window);
                            } else {
                                mouse_is_pressed = false;
                            }
                        }

                        WindowEvent::Key(key, action, _) => {
                            if input_timer.elapsed() < key_cooldown(&key) {
                                continue;
                            }

                            match key {
                                Key::S => {
                                    app_state = AppState::Simulation;
                                }

                                Key::R => {
                                    grid.randomize();
                                    grid.draw(&mut window);
                                }

                                Key::C => {
                                    grid.clear();
                                    grid.draw(&mut window);
                                }

                                _ => (), 
                            }

                            input_timer = Instant::now();
                        }
                        _ => (),
                    }
                }

                println!("User Input event time {:#?}", loop_timer.elapsed());

                // grid.draw(&mut window);
            }

            AppState::Simulation => {
                for event in events.iter() {
                    match event.value {
                        WindowEvent::Key(key, action, _) => {
                            if input_timer.elapsed() < key_cooldown(&key) {
                                continue;
                            }

                            match key {
                                Key::I => app_state = AppState::UserInput,
                                Key::Up => tick_duration /= 2,
                                Key::Down => tick_duration *= 2,
                                _ => (),
                            }

                            input_timer = Instant::now();
                        }

                        _ => (), 
                    }
                }

                println!("Simulation event time {:#?}", loop_timer.elapsed());
                
                if timer.elapsed() >= tick_duration {
                    grid.update();
                    grid.draw(&mut window);

                    timer = Instant::now();
                }
            }
        }

        println!("Full cycle time {:#?}", loop_timer.elapsed());
    }

    /*

    let rectangle = Rectangle::new(100.0, 100.0, 50.0, 50.0);
    
    let mut rect = draw_rectangle(&mut window, &rectangle);
    //let mut circ = window.add_circle(50.0);
    //circ.append_translation(&Translation2::new(200.0, 0.0));

    rect.set_color(0.0, 1.0, 0.0);
    // circ.set_color(0.0, 0.0, 1.0);

    */
}





// #[derive(Debug)]
struct Grid {
    data: [[bool; GRID_WIDTH]; GRID_HEIGHT], 
    rectangles: Vec<PlanarSceneNode>,
}

impl Grid {
    fn new() -> Self {
        Self {
            data: [[false; GRID_WIDTH]; GRID_HEIGHT], 
            rectangles: Vec::new(), 
        }
    }

    fn update(&mut self) {
        let timer = Instant::now();

        enum Update {
            Kill(usize, usize), 
            Spawn(usize, usize), 
        }

        let neighbors = |row_index: usize, col_index: usize| -> Vec<(usize, usize)> {
            let mut neighbors = Vec::new();

            for y_change in -1..=1 {
                for x_change in -1..=1 {
                    if y_change == 0 && x_change == 0 {
                        continue;
                    }

                    let neighbor_y_index = row_index.saturating_add_signed(y_change);
                    let neighbor_x_index = col_index.saturating_add_signed(x_change);

                    if neighbor_y_index >= GRID_HEIGHT || neighbor_x_index >= GRID_WIDTH {
                        continue;
                    }

                    neighbors.push((neighbor_x_index, neighbor_y_index));
                }
            }

            neighbors
        };

        let mut updates = Vec::new();

        for (row_index, row) in self.data.iter().enumerate() {
            for (col_index, element) in row.iter().enumerate() {
                let mut live_neighbor_count = 0;

                for neighbor in neighbors(row_index, col_index) {
                    if self.data[neighbor.1][neighbor.0] {
                        live_neighbor_count += 1;
                    }
                }

                if live_neighbor_count < 2 || live_neighbor_count > 3 {
                    updates.push(Update::Kill(col_index, row_index));
                } else if live_neighbor_count == 3 {
                    updates.push(Update::Spawn(col_index, row_index));
                }
            }
        }

        for update in updates {
            match update {
                Update::Kill(col_index, row_index) => self.data[row_index][col_index] = false,
                Update::Spawn(col_index, row_index) => self.data[row_index][col_index] = true,
            }
        }

        println!("Update Time: {:#?}", timer.elapsed());

    }

    fn draw(&mut self, to_window: &mut Window) {
        let timer = Instant::now();

        // let mut rect_node = draw_rectangle(to_window, &Rectangle::new(0.0, 0.0, WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32));
        // rect_node.set_color(0.0, 0.0, 0.0);
        
        // to_window.scene_mut().apply_to_scene_nodes_mut(&mut |node: &mut SceneNode| node.unlink());

        // to_window.scene_mut().unlink();

        for rect_node in self.rectangles.iter_mut() {
            to_window.remove_planar_node(rect_node);
        }

        self.rectangles = Vec::new();

        for (row_index, row) in self.data.iter().enumerate() {
            for (col_index, element) in row.iter().enumerate() {
                if *element {
                    let x = -(WINDOW_WIDTH as f32) / 2.0 + col_index as f32 * UNIT_SIZE + UNIT_SIZE / 2.0;
                    let y = (WINDOW_HEIGHT as f32) / 2.0 - row_index as f32 * UNIT_SIZE - UNIT_SIZE / 2.0;
                    let mut rect_node = draw_rectangle(to_window, &Rectangle::new(x, y, UNIT_SIZE, UNIT_SIZE));
                    rect_node.set_color(1.0, 1.0, 1.0);

                    self.rectangles.push(rect_node);
                }
            }
        }

        println!("Draw Time: {:#?}", timer.elapsed());
    }

    fn randomize(&mut self) {
        for (row_index, row) in self.data.iter_mut().enumerate() {
            for (col_index, element) in row.iter_mut().enumerate() {
                *element = rand::random::<bool>();
            }
        } 
    }

    fn clear(&mut self) {
        for (row_index, row) in self.data.iter_mut().enumerate() {
            for (col_index, element) in row.iter_mut().enumerate() {
                *element = false;
            }
        } 
    }

}

#[derive(Copy, Clone)]
struct Rectangle {
    pub x: f32, 
    pub y: f32, 
    pub width: f32, 
    pub height: f32, 
}

impl Rectangle {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x, 
            y, 
            width, 
            height, 
        }
    }
}

fn draw_rectangle(window: &mut Window, rectangle: &Rectangle) -> PlanarSceneNode {
    let mut rect_node = window.add_rectangle(rectangle.width, rectangle.height);
    let rectangle_translation = Translation2::new(rectangle.x, rectangle.y);
    rect_node.append_translation(&rectangle_translation);
    rect_node
}