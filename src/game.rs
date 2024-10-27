use std::time::SystemTime;
use fltk::enums::Color;
use rand::Rng;

pub enum GameState {
    READY,
    RUNNING,
    LOST
}

#[derive(Clone, Debug)]
enum Tetromino {
    O,
    L,
    I,
    S,
    T,
}

static SHAPES: std::sync::LazyLock<[Shape; 5]> = std::sync::LazyLock::new(|| {
    [
        Shape::new(Tetromino::I, [
            RelPoint::new(0, 0),
            RelPoint::new(0, 1),
            RelPoint::new(0, -1),
            RelPoint::new(0, -2),
        ]),
        Shape::new(Tetromino::O, [
            RelPoint::new(0, 0),
            RelPoint::new(0, 1),
            RelPoint::new(1, 0),
            RelPoint::new(1, 1),
        ]),
        Shape::new(Tetromino::S, [
            RelPoint::new(0, 0),
            RelPoint::new(0, 1),
            RelPoint::new(-1, 0),
            RelPoint::new(-1, -1),
        ]),
        Shape::new(Tetromino::L, [
            RelPoint::new(0, 1),
            RelPoint::new(1, 1),
            RelPoint::new(1, 0),
            RelPoint::new(1, -1),
        ]),
        Shape::new(Tetromino::T, [
            RelPoint::new(0, 0),
            RelPoint::new(0, 1),
            RelPoint::new(0, -1),
            RelPoint::new(1, 0),
        ]),
    ]
});

static COLORS: [Color;4] = [
    Color::Red,
    Color::Green,
    Color::Blue,
    Color::Yellow,
];

#[derive(Clone, Debug)]
pub struct RelPoint {
    pub(crate) dx: i32,
    pub(crate) dy: i32,
}

impl RelPoint {
    fn new(dx: i32, dy: i32) -> Self {
        Self { dx, dy }
    }
    fn rotate(&mut self) -> &Self {
        (self.dy, self.dx) = (self.dx, -self.dy);
        self
    }
    fn mirror(&mut self) -> &Self {
        self.dx *= -1;
        self
    }

    fn to_abs(&self, point: &Point) -> Point {
        Point::new((point.x as i32 + self.dx) as usize, (point.y as i32 + self.dy) as usize)
    }
}

#[derive(Debug)]
pub struct Point {
    pub(crate) x: usize,
    pub(crate) y: usize,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Self {
        Self {x, y}
    }

    pub fn add(&self, x: i32, y: i32) -> Point {
        Point::new((self.x as i32 + x) as usize, (self.y as i32 + y) as usize)
    }
}

#[derive(Clone, Debug)]
pub struct Shape {
    name: Tetromino,
    points: [RelPoint; 4]
}

impl Shape {
    fn new(name: Tetromino, points: [RelPoint; 4]) -> Shape {
        Self {name, points}
    }

    fn rotate(&mut self) -> &Self {
        match self.name {
            Tetromino::O => self,
            _ => {
                for point in self.points.iter_mut() {
                    point.rotate();
                }
                self
            }
        }
    }

    fn mirror(&mut self) -> &Self {
        for point in self.points.iter_mut() {
            point.mirror();
        }
        self
    }

    fn random() -> Shape {
        let idx = rand::thread_rng().gen_range(0..SHAPES.len());
        let mut shape = SHAPES[idx].clone();

        if rand::thread_rng().gen_bool(0.5) {
            shape.mirror();
        }
        for _ in 0..rand::thread_rng().gen_range(0..4) {
            shape.rotate();
        }
        shape
    }
}

pub struct PreparedShape {
    pub(crate) shape: Shape,
    pub(crate) color: Color,
}

impl PreparedShape {
    fn random() -> Self {
        let color_idx = rand::thread_rng().gen_range(0..COLORS.len());
        Self {
            shape: Shape::random(),
            color: COLORS[color_idx]
        }
    }
}

pub struct SpawnedShape {
    shape: Shape,
    loc: Point,
    pub(crate) color: Color,
}

impl SpawnedShape {
    fn random<const W: usize, const H: usize>() -> SpawnedShape {
        let PreparedShape {color, shape} = PreparedShape::random();
        SpawnedShape {
            loc: Tetris::<W, H>::starting_point(),
            shape,
            color,
        }
    }

    pub fn iter(&self) -> ShapeIter {
        ShapeIter::from_spawned(&self)
    }
}

pub struct Tetris<const W: usize, const H: usize> {
    pub field: [[Option<Color>; W]; H],
    pub next: PreparedShape,
    pub falling: SpawnedShape,
    pub state: GameState,
    pub score: usize,
    since_step: SystemTime,
    is_sped_up: bool,
}

impl<const W: usize, const H:usize> Tetris<W, H> {

    pub fn starting_point() -> Point {
        Point::new(W / 2, 5)
    }

    pub fn new() -> Tetris<W, H> {
        Tetris {
            field: [[None; W]; H],
            next: PreparedShape::random(),
            falling: SpawnedShape::random::<W, H>(),
            state: GameState::READY,
            since_step: SystemTime::now(),
            is_sped_up: false,
            score: 0,
        }
    }

    pub fn start(&mut self) {
        self.state = GameState::RUNNING;
    }

    pub fn receive_tick(&mut self) {
        let now = SystemTime::now();
        let delay = if self.is_sped_up {100} else {1000};
        if now.duration_since(self.since_step).unwrap().as_millis() > delay {
            self.step();
            self.since_step = now;
        }
    }

    pub fn receive_left(&mut self) {

        if !matches!(self.state, GameState::RUNNING) {
            return;
        }

        let future_loc = self.falling.loc.add(-1, 0);
        if !self.can_place_at(&self.falling.shape, &future_loc) {
            return;
        }
        self.falling.loc = future_loc;
    }

    pub fn receive_right(&mut self) {

        if !matches!(self.state, GameState::RUNNING) {
            return;
        }

        let future_loc = self.falling.loc.add(1, 0);
        if !self.can_place_at(&self.falling.shape, &future_loc) {
            return;
        }
        self.falling.loc = future_loc;
    }

    pub fn receive_down_press(&mut self) {

        if !matches!(self.state, GameState::RUNNING) {
            return;
        }

        self.is_sped_up = true;
    }

    pub fn receive_down_release(&mut self) {
        self.is_sped_up = false;
    }

    pub fn receive_rotate(&mut self) {

        if !matches!(self.state, GameState::RUNNING) {
            return;
        }

        let mut future_shape = self.falling.shape.clone();
        future_shape.rotate();

        if !self.can_place_at(&future_shape, &self.falling.loc) {
            return;
        }
        self.falling.shape = future_shape;
    }

    fn can_place_at(&self, shape: &Shape, loc: &Point) -> bool {
        !ShapeIter::new(&shape, &loc).any(|p| {
            p.y >= H || p.x >= W || self.field[p.y][p.x].is_some()
        })
    }

    fn is_row_packed(&self, y: usize) -> bool {
        for x in 0..W {
            if !self.field[y][x].is_some() {
                return false;
            }
        }
        true
    }

    fn destroy_full_rows(&mut self) -> usize {
        let mut moving = 0;
        for y in (0..H).rev() {
            self.field.swap(y, y + moving);
            if self.is_row_packed(y + moving) {
                self.field[y + moving].fill(None);
                moving += 1;
            }
        }
        moving
    }

    fn loose(&mut self) {
        self.state = GameState::LOST;
    }

    fn spawn_new_shape(&mut self) {
        self.falling = SpawnedShape::random::<W, H>();
        std::mem::swap(&mut self.falling.shape, &mut self.next.shape);
        std::mem::swap(&mut self.falling.color, &mut self.next.color);

        if !self.can_place_at(&self.falling.shape, &self.falling.loc) {
            self.loose();
        }
    }

    fn ground_falling_shape(&mut self) {
        self.falling.iter().for_each_mut(|p| {
            self.field[p.y][p.x] = Some(self.falling.color)
        });

        self.score += self.destroy_full_rows();
        self.spawn_new_shape();
    }

    fn step(&mut self) {

        if !matches!(self.state, GameState::RUNNING) {
            return;
        }

        let future_pos = self.falling.loc.add(0, 1);

        if self.can_place_at(&self.falling.shape, &future_pos) {
            self.falling.loc.y += 1;
            return;
        }

        self.ground_falling_shape();
    }
}

pub struct ShapeIter<'a> {
    shape: &'a Shape,
    loc: &'a Point,
    index: usize,
}

impl<'a> ShapeIter<'a> {
    pub fn new(shape: &'a Shape, loc: &'a Point) -> ShapeIter<'a> {
        ShapeIter {shape, loc, index: 0}
    }

    pub fn from_spawned(shape: &'a SpawnedShape) -> ShapeIter<'a> {
        ShapeIter::new(&shape.shape, &shape.loc)
    }

    pub fn any<T>(&self, predicate: T) -> bool where T: Fn(Point) -> bool {
        for rel_point in self.shape.points.iter() {
            if predicate(rel_point.to_abs(self.loc)) {
                return true;
            }
        }
        false
    }

    pub fn any_mut<T>(&mut self, mut predicate: T) -> bool where T: FnMut(Point) -> bool {
        for rel_point in self.shape.points.iter() {
            if predicate(rel_point.to_abs(self.loc)) {
                return true;
            }
        }
        false
    }

    pub fn for_each<T>(&self, action: T) where T: Fn(Point) {
        self.any(|p| {
            action(p);
            return false;
        });
    }

    pub fn for_each_mut<T>(&mut self, mut action: T) where T: FnMut(Point) {
        self.any_mut(|p| {
            action(p);
            return false;
        });
    }
}
