use std::collections::HashSet;

pub struct ShowGrid {
    grid: Grid,
    window_size: usize,
    in_halfstep: bool,
}

impl ShowGrid {
    pub fn new(window_size: usize) -> Self {
        Self {
            grid: Grid::new(2),
            window_size,
            in_halfstep: true, // we start with empty spaces on the board, so the first step should be filling
        }
    }

    pub fn half_step(&mut self) {
        if !self.in_halfstep {
            self.grid.expand_move();
        } else {
            self.grid.fill_spaces();
        }
        self.in_halfstep = !self.in_halfstep;
        self.check_reset();
    }

    pub fn full_step(&mut self) {
        if self.in_halfstep {
            self.grid.fill_spaces();
        }
        self.grid.expand_move();
        if !self.check_reset() {
            self.grid.fill_spaces();
        }
    }

    fn check_reset(&mut self) -> bool {
        if self.grid.radius > self.window_size as isize / 2 / 8 {
            self.grid.reset();
            self.in_halfstep = true;
            true
        } else {
            false
        }
    }

    pub fn draw(&mut self, buffer: &mut [u8]) {
        let mut canvas = Canvas(buffer, self.window_size);
        canvas.clear([0x1f, 0x1f, 0x1f, 0xff]);
        let mut x = 0isize;
        let mut y = self.grid.radius - 1;
        for f in &self.grid.vertices {
            match *f {
                Facing::Zero => {}
                Facing::One(d) => {
                    ShowGrid::draw_tile(&mut canvas, x, y, d);
                }
                Facing::Two(d, e) => {
                    ShowGrid::draw_tile(&mut canvas, x, y, d);
                    ShowGrid::draw_tile(&mut canvas, x, y, e);
                }
            }
            // go to next tile
            x += 1;
            if x + y.abs() >= self.grid.radius {
                y -= 1;
                x = -self.grid.radius + y.abs() + 1;
            }
        }
    }

    fn draw_tile(canvas: &mut Canvas, x: isize, y: isize, dir: Direction) {
        // colors for drawing, kept in static storage
        static RED: [u8; 4] = [0xf7, 0x1e, 0x1e, 0xff];
        static GREEN: [u8; 4] = [0x30, 0xf7, 0x1e, 0xff];
        static BLUE: [u8; 4] = [0x1e, 0x71, 0xf7, 0xff];
        static YELLOW: [u8; 4] = [0xf7, 0xd7, 0x1e, 0xff];
        static GREY: [u8; 4] = [0x4a, 0x4a, 0x4a, 0xff];
        // grid coordinates of logical drawing square extending to lower right of (x, y)-point
        let grid_x = (canvas.1 as isize / 2 / 8 - 1 + x) as usize;
        let grid_y = (canvas.1 as isize / 2 / 8 - 1 - y) as usize;
        // each logical square consists of 8x8 pixels
        let pixel_x = 8 * grid_x;
        let pixel_y = 8 * grid_y;

        let (corner, size, color) = match dir {
            Direction::Up => {
                let corner = (pixel_x - 8, pixel_y);
                let size = (16, 8);
                (corner, size, BLUE)
            }
            Direction::Down => {
                let corner = (pixel_x - 8, pixel_y - 8);
                let size = (16, 8);
                (corner, size, RED)
            }
            Direction::Left => {
                let corner = (pixel_x, pixel_y - 8);
                let size = (8, 16);
                (corner, size, GREEN)
            }
            Direction::Right => {
                let corner = (pixel_x - 8, pixel_y - 8);
                let size = (8, 16);
                (corner, size, YELLOW)
            }
        };
        canvas.bound(corner.0, corner.1, size.0, size.1, GREY);
        canvas.fill(corner.0 + 1, corner.1 + 1, size.0 - 1, size.1 - 1, color);
    }
}

struct Canvas<'a>(&'a mut [u8], usize);

impl<'a> Canvas<'a> {
    // fn set(&mut self, x: usize, y: usize, color: [u8; 4]) {
    //     self.0[(4 * (y * self.1 + x))..(4 * (y * self.1 + x + 1))].copy_from_slice(&color);
    // }

    fn bound(&mut self, x: usize, y: usize, w: usize, h: usize, color: [u8; 4]) {
        for pixel_val in
            self.0[(4 * (y * self.1 + x))..(4 * (y * self.1 + x + w))].chunks_exact_mut(4)
        {
            pixel_val.copy_from_slice(&color);
        }
        for pixel_val in self.0
            [(4 * ((y + h - 1) * self.1 + x))..(4 * ((y + h - 1) * self.1 + x + w))]
            .chunks_exact_mut(4)
        {
            pixel_val.copy_from_slice(&color);
        }
        for row in (y + 1)..(y + h - 1) {
            self.0[(4 * (row * self.1 + x))..(4 * (row * self.1 + x + 1))].copy_from_slice(&color);
            self.0[(4 * (row * self.1 + x + w - 1))..(4 * (row * self.1 + x + w))]
                .copy_from_slice(&color);
        }
    }

    fn fill(&mut self, x: usize, y: usize, w: usize, h: usize, color: [u8; 4]) {
        for row in y..(y + h) {
            for pixel_val in
                self.0[(4 * (row * self.1 + x))..(4 * (row * self.1 + x + w))].chunks_exact_mut(4)
            {
                pixel_val.clone_from_slice(&color);
            }
        }
    }

    fn clear(&mut self, color: [u8; 4]) {
        self.fill(0, 0, self.1, self.1, color);
    }
}

pub(crate) struct Grid {
    pub radius: isize,
    vertices: Vec<Facing>,
}

impl std::fmt::Debug for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rows = 2 * (self.radius - 1) as usize;
        let mut data = vec![vec!['·'; rows]; rows];

        let mut x = 0isize;
        let mut y = self.radius - 1;
        for f in &self.vertices {
            // only process non-edge vertices
            let mut show_dir = |dir: Direction| {
                let grid_x = (self.radius - 1 + x) as usize;
                let grid_y = (self.radius - 1 - y) as usize;
                match dir {
                    Direction::Up => {
                        data[grid_y][grid_x] = '↑';
                        data[grid_y][grid_x - 1] = '↑';
                    }
                    Direction::Down => {
                        data[grid_y - 1][grid_x] = '↓';
                        data[grid_y - 1][grid_x - 1] = '↓';
                    }
                    Direction::Left => {
                        data[grid_y][grid_x] = '←';
                        data[grid_y - 1][grid_x] = '←';
                    }
                    Direction::Right => {
                        data[grid_y][grid_x - 1] = '→';
                        data[grid_y - 1][grid_x - 1] = '→';
                    }
                }
            };
            match *f {
                Facing::Zero => {}
                Facing::One(d) => {
                    show_dir(d);
                }
                Facing::Two(d, e) => {
                    show_dir(d);
                    show_dir(e);
                }
            }
            // go to next tile
            x += 1;
            if x + y.abs() >= self.radius as isize {
                y -= 1;
                x = -self.radius + y.abs() + 1;
            }
        }
        let lines = data
            .into_iter()
            .map(|x| x.into_iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "\n{}\n", lines)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Facing {
    Zero,
    One(Direction),
    Two(Direction, Direction),
}

impl Facing {
    pub fn add_dir(&mut self, dir: Direction) {
        *self = match &self {
            Facing::Zero => Facing::One(dir),
            Facing::One(x) => Facing::Two(*x, dir),
            Facing::Two(_, _) => panic!("too many directions added to single vertex"),
        }
    }
}

impl Default for Facing {
    fn default() -> Self {
        Facing::Zero
    }
}

impl Grid {
    pub(crate) fn new(radius: isize) -> Self {
        Self {
            radius,
            vertices: vec![Facing::default(); (1 + 2 * radius * (radius - 1)) as usize],
        }
    }

    pub fn reset(&mut self) {
        *self = Grid::new(2);
    }

    fn get(&self, x: isize, y: isize) -> Facing {
        if x.abs() + y.abs() >= self.radius {
            Facing::Zero
        } else {
            let i = self.new_data_index((x, y));
            self.vertices[i as usize]
        }
    }

    fn new_data_index(&self, mut index: (isize, isize)) -> isize {
        let mut i = 0;
        if index.1 < 0 {
            i += self.radius * self.radius;
            index.1 *= -1;
            i += (self.radius - 1) * (self.radius - 1)
                - (self.radius - index.1) * (self.radius - index.1);
        } else {
            let n_occupied = self.radius - 1 - index.1;
            i += n_occupied * n_occupied;
        }
        i + self.radius - index.1 - 1 + index.0
    }

    pub fn expand_move(&mut self) {
        // initialize the new, expanded grid
        let mut new_grid = Grid::new(self.radius + 1);
        let old_r = self.radius;
        for y in -(old_r - 0)..=(old_r - 0) {
            for x in -(old_r - y.abs() - 1)..=(old_r - y.abs() - 1) {
                use {Direction::*, Facing::*};
                match self[(x, y)] {
                    One(Up) => new_grid[(x, y + 1)].add_dir(Up),
                    One(Down) => new_grid[(x, y - 1)].add_dir(Down),
                    One(Left) => new_grid[(x - 1, y)].add_dir(Left),
                    One(Right) => new_grid[(x + 1, y)].add_dir(Right),
                    _ => (), // Case 'Two' leads to annihilation
                }
            }
        }
        *self = new_grid;
    }

    pub fn fill_spaces(&mut self) {
        // let rng = fastrand::Rng::new();
        let mut x = 0isize;
        let mut y = self.radius - 1;
        // set of all centers of empty 2x2 tiles (may overlap)
        let mut empty_tiles = HashSet::with_capacity(self.radius as usize - 1);
        for _ in &self.vertices {
            // only process non-edge vertices
            if x.abs() + y.abs() < self.radius - 1 {
                use Direction::*;
                use Facing::*;
                if let (
                    Zero | One(Left | Down),
                    Zero | One(Left | Up),
                    Zero | One(Right | Up),
                    Zero | One(Right | Down),
                    Zero | One(Down),
                    Zero | One(Up),
                    Zero | One(Right),
                    Zero | One(Left),
                    Zero,
                ) = (
                    self.get(x + 1, y + 1),
                    self.get(x + 1, y - 1),
                    self.get(x - 1, y - 1),
                    self.get(x - 1, y + 1),
                    self[(x, y + 1)],
                    self[(x, y - 1)],
                    self[(x - 1, y)],
                    self[(x + 1, y)],
                    self[(x, y)],
                ) {
                    empty_tiles.insert((x, y));
                }
            }
            // go to next tile
            x += 1;
            if x + y.abs() >= self.radius as isize {
                y -= 1;
                x = -self.radius + y.abs() + 1;
            }
        }

        let mut fillable = HashSet::new();
        // println!("{:?}", &empty_tiles);
        while !empty_tiles.is_empty() {
            // let mut fillable = HashSet::new();
            for (x, y) in &empty_tiles {
                if !(empty_tiles.contains(&(*x - 1, *y)) && empty_tiles.contains(&(*x + 1, *y)))
                    && !(empty_tiles.contains(&(*x, *y - 1)) && empty_tiles.contains(&(*x, *y + 1)))
                {
                    fillable.insert((*x, *y));
                }
            }
            for (x, y) in fillable.drain() {
                if empty_tiles.contains(&(x, y)) {
                    if fastrand::bool() {
                        self[(x - 1, y)].add_dir(Direction::Left);
                        self[(x + 1, y)].add_dir(Direction::Right);
                    } else {
                        self[(x, y + 1)].add_dir(Direction::Up);
                        self[(x, y - 1)].add_dir(Direction::Down);
                    }
                    empty_tiles.remove(&(x, y));
                    empty_tiles.remove(&(x - 1, y));
                    empty_tiles.remove(&(x + 1, y));
                    empty_tiles.remove(&(x, y - 1));
                    empty_tiles.remove(&(x, y + 1));
                    empty_tiles.remove(&(x + 1, y + 1));
                    empty_tiles.remove(&(x + 1, y - 1));
                    empty_tiles.remove(&(x - 1, y - 1));
                    empty_tiles.remove(&(x - 1, y + 1));
                    // dbg!(&self);
                    // println!("{:?}", &empty_tiles);
                }
            }
        }
    }
}

impl std::ops::Index<(isize, isize)> for Grid {
    type Output = Facing;

    fn index(&self, index: (isize, isize)) -> &Self::Output {
        assert!(index.0.abs() + index.1.abs() < self.radius);
        let i = self.new_data_index(index) as usize;
        &self.vertices[i]
    }
}

impl std::ops::IndexMut<(isize, isize)> for Grid {
    fn index_mut(
        &mut self,
        index: (isize, isize),
    ) -> &mut <Self as std::ops::Index<(isize, isize)>>::Output {
        assert!(index.0.abs() + index.1.abs() < self.radius);
        let i = self.new_data_index(index) as usize;
        &mut self.vertices[i]
    }
}
