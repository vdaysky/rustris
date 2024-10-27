mod game;

use std::cell::{RefCell};
use std::rc::Rc;
use fltk::*;
use fltk::app::{App, Sender};
use fltk::button::Button;
use fltk::enums::{Align, Color, Event, Font, Key};

use fltk::group::{Pack};
use fltk::prelude::{GroupExt, ImageExt, WidgetBase, WidgetExt};
use fltk::window::{DoubleWindow, Window};
use game::{ShapeIter, GameState, Point, Tetris};

const CELL_SIZE: i32 = 40;
const CANVAS_X: i32 = 0;
const CANVAS_Y: i32 = 40;

const WINDOW_W: i32 = 680;
const WINDOW_H: i32 = 880;

const KEY_A: Key = Key::from_char('a');
const KEY_D: Key = Key::from_char('d');
const KEY_S: Key = Key::from_char('s');
const KEY_W: Key = Key::from_char('w');

#[derive(Copy, Clone)]
enum Page {
    Menu,
    Game,
}

fn main() {
    let app = App::default();
    let mut wind = Window::new(100, 100, WINDOW_W, WINDOW_H, "Tetris!");
    let mut pack = Pack::new(0, 0, wind.width(), wind.height(), "");
    let (sender, receiver) = app::channel::<Page>();

    setup_menu(&mut wind, &mut pack, sender);

    wind.end();
    wind.show();

    while app.wait() {
        match receiver.recv() {
            Some(Page::Game) => {
                setup_game(Tetris::<10, 20>::new(), &mut wind, &mut pack, sender);
                app::sleep(0.016);
            },
            Some(Page::Menu) => {
                setup_menu(&mut wind, &mut pack, sender);
                app::sleep(0.016);
            }
            _ => ()
        }
        wind.redraw();
    }
}

fn setup_menu(wind: &mut DoubleWindow, pack: &mut Pack, sender: Sender<Page>) {
    pack.clear();
    pack.begin();

    Button::new(100, 100, 200, 40, "Start!").emit(sender, Page::Game);

    pack.end();
    pack.show();
    pack.draw(|_|{});
    pack.redraw();
    wind.handle(|_, _| false);
}

fn draw_cell(x: usize, y: usize, color: Color){
    draw::set_draw_color(color);
    draw::draw_rectf(x as i32 * CELL_SIZE + CANVAS_X, y as i32 * CELL_SIZE + CANVAS_Y, CELL_SIZE, CELL_SIZE);
    draw::set_draw_color(Color::from_rgb(148, 151, 192));
    draw::draw_rectf(x as i32 * CELL_SIZE + CANVAS_X, y as i32 * CELL_SIZE + CANVAS_Y, 3, CELL_SIZE-3);
    draw::draw_rectf(x as i32 * CELL_SIZE + CANVAS_X, y as i32 * CELL_SIZE + CANVAS_Y, CELL_SIZE-3, 3);
}


fn draw_game<const W: usize, const H: usize>(tetris: Rc<RefCell<Tetris<W, H>>>) {

    {
        let tetris_mut = tetris.borrow_mut();

        tetris_mut.falling.iter().for_each(
            |p| draw_cell(p.x, p.y, tetris_mut.falling.color)
        );

        let next_shape_display_loc = Point::new(W + 3, 3);
        ShapeIter::new(&tetris_mut.next.shape, &next_shape_display_loc).for_each(
            |p| draw_cell(p.x, p.y, tetris_mut.next.color)
        );

        for x in 0..W {
            for y in 0..H {
                match tetris_mut.field[y][x] {
                    Some(color) => draw_cell(x, y, color),
                    _ => {}
                }
            }
        }
    }

    {
        draw::set_font(Font::Courier, 44);
        draw::set_draw_color(Color::Red);
        draw::draw_text2(
            &*format!("Score: {score}", score = tetris.borrow().score),
            W as i32 * CELL_SIZE + 10,
            CANVAS_Y + 6 * CELL_SIZE,
            10 * CELL_SIZE,
            50,
            Align::Left
        );
    }

    if matches!(tetris.borrow().state, GameState::LOST) {
        draw::set_font(Font::Courier, 50);
        draw::set_draw_color(Color::Red);
        draw::draw_text2("Game Over", 0, CANVAS_Y + 50, 10 * CELL_SIZE, 50, Align::Center);
    }
}

fn setup_game<const W: usize, const H: usize>(tetris: Tetris<W, H>, wind: &mut DoubleWindow, pack: &mut Pack, sender: Sender<Page>) {
    pack.clear();
    pack.begin();

    Button::new(0, 0, 50, 40, "Back!").emit(sender, Page::Menu);

    pack.end();
    pack.show();

    let mut tetris_rc = Rc::new(RefCell::new(tetris));
    tetris_rc.borrow_mut().start();

    let tetris_rc1 = tetris_rc.clone();
    let tetris_rc2 = tetris_rc.clone();

    pack.draw(move |_| {
        draw::set_draw_color(Color::Black);
        draw::draw_rectf(0, CANVAS_Y, W as i32 * CELL_SIZE, H as i32 * CELL_SIZE);

        tetris_rc1.borrow_mut().receive_tick();
        draw_game(tetris_rc1.clone());
    });

    wind.handle(
        move |_, ev| {
            match ev {
                Event::Focus => true,
                Event::KeyUp => {
                    let key = app::event_key();
                    if key == KEY_S {
                        tetris_rc2.borrow_mut().receive_down_release();
                    }
                    return true;
                },
                Event::KeyDown => {
                    match app::event_key() {
                        KEY_A => tetris_rc2.borrow_mut().receive_left(),
                        KEY_D => tetris_rc2.borrow_mut().receive_right(),
                        KEY_S => tetris_rc2.borrow_mut().receive_down_press(),
                        KEY_W => tetris_rc2.borrow_mut().receive_rotate(),
                        _ => {},
                    }
                    true
                },
                _ => false,
            }
        }
    );
}
