use std::collections::HashMap;

use ::rand::{distributions::Standard, prelude::Distribution, Rng};
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "infinite klondike".to_owned(),
        high_dpi: false,
        ..Default::default()
    }
}

fn draw_texture_box(texture: Texture2D, x: f32, y: f32, color: Color, src: Rect) {
    draw_texture_ex(
        texture,
        x,
        y,
        color,
        DrawTextureParams {
            dest_size: Some(Vec2::new(44.0, 64.0)),
            source: Some(src),
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: None,
        },
    );
}

fn draw_atlas_item(atlas: Texture2D, x: f32, y: f32, offset: f32) {
    draw_texture_box(
        atlas,
        x,
        y,
        WHITE,
        Rect {
            x: offset,
            y: 0.0,
            w: 22.0,
            h: 32.0,
        },
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Suit {
    Club,
    Diamond,
    Heart,
    Spade,
}

impl Suit {
    const fn get_x(&self) -> f32 {
        match self {
            Suit::Club => 330.0,
            Suit::Diamond => 352.0,
            Suit::Heart => 374.0,
            Suit::Spade => 396.0,
        }
    }
    const fn is_red(&self) -> bool {
        match self {
            Suit::Club | Suit::Spade => false,
            Suit::Diamond | Suit::Heart => true,
        }
    }
    const fn get_color(&self) -> Color {
        match self {
            Suit::Club | Suit::Spade => WHITE,
            Suit::Diamond | Suit::Heart => RED,
        }
    }
}

impl Distribution<Suit> for Standard {
    fn sample<R: ::rand::Rng + ?Sized>(&self, rng: &mut R) -> Suit {
        match rng.gen_range(0..=3) {
            0 => Suit::Club,
            1 => Suit::Diamond,
            2 => Suit::Heart,
            _ => Suit::Spade,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Pip(u8);

impl Pip {
    const fn is_ace(self) -> bool {
        self.0 == 0
    }
    fn get_x(&self) -> f32 {
        44.0 + 22.0 * self.0 as f32
    }
    fn can_drop_on(&self, other: &Pip) -> bool {
        self.0 + 1 == other.0
    }
}

#[derive(Debug, Clone)]
struct Card {
    suit: Suit,
    inner: Pip,
}

impl Distribution<Card> for Standard {
    fn sample<R: ::rand::Rng + ?Sized>(&self, rng: &mut R) -> Card {
        Card {
            suit: rng.gen(),
            inner: Pip(rng.gen_range(0..13)),
        }
    }
}

impl Card {
    fn draw(&self, atlas: Texture2D, x: f32, y: f32) {
        let color = self.suit.get_color();
        draw_texture_box(atlas, x, y, WHITE, Rect::new(0.0, 0.0, 22.0, 32.0));
        draw_texture_box(
            atlas,
            x,
            y,
            color,
            Rect::new(self.suit.get_x(), 0.0, 22.0, 32.0),
        );
        draw_texture_box(
            atlas,
            x,
            y,
            color,
            Rect::new(self.inner.get_x(), 0.0, 22.0, 32.0),
        );
    }
    fn can_drop_on(&self, other: &Card) -> bool {
        self.inner.can_drop_on(&other.inner) && self.suit.is_red() != other.suit.is_red()
    }
}

struct Stack {
    visible: Vec<Card>,
    under: u32,
}

struct State {
    grabbed_stack: Vec<Card>,
    grabbed_stack_row: usize,
    tableau: Vec<Stack>,
    foundations: HashMap<usize, Card>,
    camera: Vec2,
}

impl State {
    const ROW_WIDTH: f32 = 48.0;
    const TABLEAU_Y_OFFSET: f32 = 68.0;
    const FOUNDATION_X_OFFSET: f32 = Self::ROW_WIDTH * 3.0;
    fn new() -> Self {
        let mut tableau = Vec::new();
        let mut rng = ::rand::thread_rng();
        for x in 0..50 {
            let stack = Stack {
                visible: vec![rng.gen()],
                under: x,
            };
            tableau.push(stack)
        }

        let w = screen_width();
        let shown_cards = 7.0;
        let camera = Vec2::new(w - (shown_cards - 1.0) * 48.0, 2.0);
        State {
            grabbed_stack: Vec::new(),
            tableau,
            foundations: HashMap::new(),
            grabbed_stack_row: 0,
            camera,
        }
    }
    fn get_row_over_mouse(&self) -> Option<usize> {
        let (x, _) = mouse_position();
        let x = x - self.camera.x + Self::ROW_WIDTH;
        if x < 0.0 {
            return None;
        }
        Some((x / Self::ROW_WIDTH) as usize)
    }
    fn draw(&self, atlas: Texture2D) {
        let min = -self.camera.x / 48.0;
        let w = screen_width();
        let visible = (w - self.camera.x) as usize / 48 + 2;
        let tableau_slice = &self.tableau[min as usize..visible];
        let camera_offset_x = if self.camera.x > 0.0 {
            self.camera.x
        } else {
            self.camera.x % 48.0
        };
        for (x, stack) in tableau_slice.iter().enumerate() {
            for y in 0..stack.under {
                draw_atlas_item(
                    atlas,
                    48.0 * (x as f32 - 1.0) + camera_offset_x,
                    16.0 * y as f32 + Self::TABLEAU_Y_OFFSET + self.camera.y,
                    22.0,
                )
            }
            if stack.under == 0 && stack.visible.is_empty() {
                // draw empty
                draw_atlas_item(
                    atlas,
                    48.0 * (x as f32 - 1.0) + camera_offset_x,
                    Self::TABLEAU_Y_OFFSET + self.camera.y,
                    418.0,
                )
            } else {
                for (n, card) in stack.visible.iter().enumerate() {
                    card.draw(
                        atlas,
                        48.0 * (x as f32 - 1.0) + camera_offset_x,
                        16.0 * (n + stack.under as usize) as f32
                            + Self::TABLEAU_Y_OFFSET
                            + self.camera.y,
                    );
                }
            }
        }
        // draw foundation
        let foundation_min = ((Self::FOUNDATION_X_OFFSET - self.camera.x) / 48.0 - 5.0) as usize;
        let foundation_camera_x_offset = if self.camera.x < -Self::FOUNDATION_X_OFFSET {
            self.camera.x % 48.0 + Self::ROW_WIDTH
        } else {
            self.camera.x + Self::FOUNDATION_X_OFFSET
        };
        for x in foundation_min..visible {
            let local_x = x - foundation_min;
            if let Some(card) = self.foundations.get(&x) {
                card.draw(
                    atlas,
                    48.0 * (local_x as f32 - 1.0) + foundation_camera_x_offset,
                    self.camera.y,
                )
            } else {
                draw_atlas_item(
                    atlas,
                    48.0 * (local_x as f32 - 1.0) + foundation_camera_x_offset,
                    self.camera.y,
                    418.0,
                )
            }
        }
        for (n, card) in self.grabbed_stack.iter().enumerate() {
            let (x, y) = mouse_position();
            let x = (x / 2.0).floor() * 2.0;
            let y = (y / 2.0).floor() * 2.0;
            card.draw(atlas, x, y + (16.0 * n as f32));
        }
        // debug!("{:?}", Self::get_row_over_mouse());
    }
    fn is_mouse_on_foundation(&self) -> bool {
        let (_, y) = mouse_position();
        let y = y - self.camera.y;
        y < Self::TABLEAU_Y_OFFSET
    }
    fn on_click(&mut self) {
        if self.grabbed_stack.is_empty() {
            // nothing grabbed
            if let Some(row_over) = self.get_row_over_mouse() {
                // calculate where the split is (vertically)
                let (_, y) = mouse_position();
                let y = (y - self.camera.y - Self::TABLEAU_Y_OFFSET) as usize / 16;
                if let Some(visible_idx) =
                    y.checked_sub(self.tableau[row_over].under.try_into().unwrap())
                {
                    self.grabbed_stack_row = row_over;
                    if visible_idx >= self.tableau[row_over].visible.len() {
                        // only pickup the top card
                        if let Some(card) = self.tableau[row_over].visible.pop() {
                            self.grabbed_stack.push(card);
                        }
                    } else {
                        self.grabbed_stack
                            .extend(self.tableau[row_over].visible.drain(visible_idx..))
                    }
                }
            }
        } else {
            // drop grabbed stack on other stack
            if let Some(row_over) = self.get_row_over_mouse() {
                if self.is_mouse_on_foundation() && self.grabbed_stack.len() == 1 {
                    let grabbed_card = &self.grabbed_stack[0];
                    if let Some(foundation_index) = row_over.checked_sub(3) {
                        if let Some(card) = self.foundations.get(&foundation_index) {
                            if card.suit == grabbed_card.suit
                                && card.inner.0 + 1 == grabbed_card.inner.0
                            {
                                self.foundations
                                    .insert(foundation_index, self.grabbed_stack.pop().unwrap());
                                if self.tableau[self.grabbed_stack_row].under > 0
                                    && self.tableau[self.grabbed_stack_row].visible.is_empty()
                                {
                                    self.tableau[self.grabbed_stack_row]
                                        .visible
                                        .push(::rand::random());
                                    self.tableau[self.grabbed_stack_row].under -= 1;
                                }
                            }
                        } else {
                            if grabbed_card.inner.is_ace() {
                                self.foundations
                                    .insert(foundation_index, self.grabbed_stack.pop().unwrap());
                                if self.tableau[self.grabbed_stack_row].under > 0
                                    && self.tableau[self.grabbed_stack_row].visible.is_empty()
                                {
                                    self.tableau[self.grabbed_stack_row]
                                        .visible
                                        .push(::rand::random());
                                    self.tableau[self.grabbed_stack_row].under -= 1;
                                }
                            }
                        }
                    } else {
                        self.tableau[self.grabbed_stack_row]
                            .visible
                            .append(&mut self.grabbed_stack)
                    }
                } else {
                    let stack = &mut self.tableau[row_over];
                    if self
                        .grabbed_stack
                        .first()
                        .zip(stack.visible.last())
                        .map(|(grabbed, target)| grabbed.can_drop_on(target))
                        .unwrap_or(false)
                        || stack.under == 0 && stack.visible.is_empty()
                    {
                        stack.visible.append(&mut self.grabbed_stack);
                        // success, deal with the grabbed stack
                        if self.tableau[self.grabbed_stack_row].under > 0
                            && self.tableau[self.grabbed_stack_row].visible.is_empty()
                        {
                            self.tableau[self.grabbed_stack_row]
                                .visible
                                .push(::rand::random());
                            self.tableau[self.grabbed_stack_row].under -= 1;
                        }
                    } else {
                        self.tableau[self.grabbed_stack_row]
                            .visible
                            .append(&mut self.grabbed_stack)
                    }
                }
            } else {
                self.tableau[self.grabbed_stack_row]
                    .visible
                    .append(&mut self.grabbed_stack)
            }
        }
    }
    fn generate_new(&mut self) {
        let w = screen_width();
        let visible = (w - self.camera.x) as usize / 48 + 2;
        if visible > self.tableau.len() {
            for height in self.tableau.len()..visible {
                self.tableau.push(Stack {
                    visible: vec![::rand::random()],
                    under: height as u32,
                })
            }
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let atlas = load_texture("cards.png").await.expect("could not find cards.png");
    atlas.set_filter(FilterMode::Nearest);
    let mut state = State::new();
    let mut old_pos = mouse_position();
    loop {
        clear_background(BLACK);

        state.draw(atlas);

        //draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        //draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        //draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);
        //draw_text("HELLO", 20.0, 20.0, 20.0, DARKGRAY);
        if is_mouse_button_pressed(MouseButton::Left) {
            state.on_click();
        }
        if is_mouse_button_down(MouseButton::Right) {
            if is_mouse_button_pressed(MouseButton::Right) {
                old_pos = mouse_position();
            }
            let new_pos = mouse_position();
            let dx = new_pos.0 - old_pos.0;
            let dy = new_pos.1 - old_pos.1;
            state.camera.x += dx;
            state.camera.y += dy;
            old_pos = new_pos;
            state.generate_new();
        }
        next_frame().await
    }
}
