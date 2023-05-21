mod input;
mod status;
mod textures;

use super::*;

use crate::coordinations::*;
pub use status::{
    Active, Cell, Controller, Grid, LivesLeft, MinesLeft, NonActive, Statistics, Status,
    StatusGenerator, UserStat,
};

use piston_window as pw;
use pw::Transformed;

use itertools::iproduct;

use std::collections::HashMap;

pub static WINDOW_DEFAULT_TITLE: &str = "Sioux minesweeper, in Rust!";
const WINDOW_DEFAULT_WIDTH: u32 = 640;
const WINDOW_DEFAULT_HEIGHT: u32 = 480;

const DUMMY_PLAYER_ID: PlayerID = PlayerID(0);

static BACKGROUND_COLOR: Color = WHITE;
static WHITE: Color = [1.0, 1.0, 1.0, 1.0];
static GREEN: Color = [0.5, 1.0, 0.5, 1.0];
static BLACK: Color = [0.0, 0.0, 0.0, 1.0];
static BLUE: Color = [0.5, 0.5, 1.0, 1.0];
static YELLOW: Color = [1.0, 1.0, 0.0, 1.0];
static PURPLE: Color = [0.7, 0.2, 0.7, 1.0];
static ORANGE: Color = [1.0, 0.5, 0.0, 1.0];
static BROWN: Color = [0.6, 0.2, 0.2, 1.0];
static CYAN: Color = [0.0, 1.0, 1.0, 1.0];
static MAGENTA: Color = [1.0, 0.2, 1.0, 1.0];
static GRAY: Color = [0.5, 0.5, 0.5, 1.0];
static LIGHT_GREEN: Color = [0.2, 0.9, 0.2, 1.0];

static FONT_DATA: &[u8] = include_bytes!("courier.ttf");

type Color = [f32; 4];

#[derive(Clone)]
struct Canvas {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

struct CanvasSize {
    w: f64,
    h: f64,
}

struct CanvasRatio {
    w_over_h: f64,
}

impl From<input::DrawSize> for CanvasSize {
    fn from(s: input::DrawSize) -> Self {
        let input::DrawSize { width, height } = s;
        Self {
            w: width as f64,
            h: height as f64,
        }
    }
}

impl From<[u32; 2]> for CanvasSize {
    fn from(s: [u32; 2]) -> Self {
        Self {
            w: s[0] as f64,
            h: s[1] as f64,
        }
    }
}

type Transform = [[f64; 3]; 2];

fn rect_transform(t: &Transform, canvas: &Canvas, canvas_size: &CanvasSize) -> Transform {
    t.scale(canvas_size.w, canvas_size.h)
        .trans(canvas.x, canvas.y)
        .scale(canvas.w, canvas.h)
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum FitHorizontal {
    Left,
    Center,
    Right,
}
#[derive(Clone, Copy)]
#[allow(dead_code)]
enum FitVertical {
    Bottom,
    Center,
    Top,
}
#[derive(Clone, Copy)]
struct Fit(FitHorizontal, FitVertical);

impl Fit {
    fn rebuild_canvas(
        &self,
        canvas: &Canvas,
        canvas_size: &CanvasSize,
        canvas_target_ratio: CanvasRatio,
    ) -> Canvas {
        let Fit(hor, ver) = *self;
        #[allow(non_snake_case)]
        let W = canvas_size.w * canvas.w;
        #[allow(non_snake_case)]
        let H = canvas_size.h * canvas.h;
        let CanvasRatio { w_over_h } = canvas_target_ratio;
        let mut r: Canvas = canvas.clone();
        if W > w_over_h * H {
            r.w = canvas.h * w_over_h * canvas_size.h / canvas_size.w;
            let dw = canvas.w - r.w;
            match hor {
                FitHorizontal::Left => {}
                FitHorizontal::Right => {
                    r.x += dw;
                }
                FitHorizontal::Center => {
                    r.x += dw * 0.5;
                }
            }
        } else {
            r.h = canvas.w * canvas_size.w / (canvas_size.h * w_over_h);
            let dh = canvas.h - r.h;
            match ver {
                FitVertical::Bottom => {}
                FitVertical::Top => {
                    r.y += dh;
                }
                FitVertical::Center => {
                    r.y += dh * 0.5;
                }
            }
        }
        r
    }
}

fn construct_canvas_and_transform(
    context_transform: &Transform,
    coords: &Coordinations,
    canvas: &Canvas,
    canvas_size: &CanvasSize,
    fit: Option<Fit>,
) -> (Canvas, Transform) {
    let canvas_target_ratio = CanvasRatio {
        w_over_h: coords.w_over_h(),
    };
    let canvas = fit.map_or(canvas.clone(), |fit| {
        fit.rebuild_canvas(canvas, canvas_size, canvas_target_ratio)
    });
    let transform = rect_transform(context_transform, &canvas, canvas_size);
    (canvas, transform)
}

#[allow(clippy::too_many_arguments)]
fn identify_cell(
    cursor_pos: &input::CursorPosition,
    draw_size: &input::DrawSize,
    context_transform: &Transform,
    coords: &Coordinations,
    canvas: &Canvas,
    fit: Option<Fit>,
) -> Option<Coord> {
    let canvas_size: CanvasSize = (*draw_size).into();
    let (_, transform) =
        construct_canvas_and_transform(context_transform, coords, canvas, &canvas_size, fit);

    let inv = pw::math::invert(transform);
    let [x, y] = pw::math::transform_pos(inv, (*cursor_pos).into());
    ((0. ..=1.).contains(&x) && (0. ..=1.).contains(&y)).then(|| {
        let x = x * coords.columns() as f64;
        let y = y * coords.rows() as f64;
        Coord {
            x: x as u32,
            y: y as u32,
        }
    })
}

type GLTexture = pw::Texture<gfx_device_gl::Resources>;
type GLResources = gfx_device_gl::Resources;

#[allow(clippy::too_many_arguments)]
fn draw_grid<'a, G, P>(
    context: &pw::Context,
    graphics: &mut G,
    coords: &Coordinations,
    canvas: &Canvas,
    draw_size: &input::DrawSize,
    fit: Option<Fit>,
    players_and_textures: P,
) where
    G: pw::Graphics<Texture = GLTexture>,
    P: Fn(&Coord) -> (Option<&'a GLTexture>, Option<PlayerID>),
{
    let canvas_size: CanvasSize = (*draw_size).into();
    let (canvas, transform) =
        construct_canvas_and_transform(&context.transform, coords, canvas, &canvas_size, fit);
    let h = 1. / coords.rows() as f64;
    let w = 1. / coords.columns() as f64;
    // draw fill
    for row in 0..coords.rows() {
        let y = row as f64 * h;
        for column in 0..coords.columns() {
            let x = column as f64 * w;
            let (_texture, player) = players_and_textures(&Coord { x: column, y: row });
            if let Some(player) = player {
                let rectangle = pw::Rectangle::new(player.color());
                let dims = [x, y, w, h];
                rectangle.draw(dims, &context.draw_state, transform, graphics);
            }
        }
    }
    // draw textures
    {
        let w = canvas.w / coords.columns() as f64;
        let h = canvas.h / coords.rows() as f64;
        let rect_image: pw::Image = pw::Image::new().rect(pw::rectangle::square(0.0, 0.0, 1.0));
        for row in 0..coords.rows() {
            let y = canvas.y + row as f64 * h;
            for column in 0..coords.columns() {
                let (texture, _player) = players_and_textures(&Coord { x: column, y: row });
                if let Some(texture) = texture {
                    let x = canvas.x + column as f64 * w;
                    let canvas = Canvas { x, y, w, h };
                    let transform = rect_transform(&context.transform, &canvas, &canvas_size);
                    rect_image.draw(texture, &context.draw_state, transform, graphics);
                }
            }
        }
    }
    // draw lines
    for row in 0..coords.rows() + 1 {
        let y = row as f64 * h;
        let line = [0., y, 1., y];
        pw::line(BLACK, 0.05 * h, line, transform, graphics);
    }
    for column in 0..coords.columns() + 1 {
        let x = column as f64 * w;
        let line = [x, 0., x, 1.];
        pw::line(BLACK, 0.05 * w, line, transform, graphics);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_active_text<'a, G, P>(
    context: &pw::Context,
    graphics: &mut G,
    coords: &Coordinations,
    canvas: &Canvas,
    canvas_size: &CanvasSize,
    fit: Option<Fit>,
    glyphs: &mut pw::Glyphs,
    v: &rusttype::VMetrics,
    players: P,
    lives_left: status::LivesLeft,
    mines_left: status::MinesLeft,
) where
    G: pw::Graphics<Texture = GLTexture>,
    P: Fn(&Coord) -> Option<Player<'a>>,
{
    let (mines_text, lives_text) = {
        let status::MinesLeft(mines_left) = mines_left;
        let status::LivesLeft(lives_left) = lives_left;
        let mines_text = format!("Mines: {}", mines_left);
        let lives_text = format!("Lives: {}", lives_left);
        (mines_text, lives_text)
    };
    let texts = {
        let players: HashMap<PlayerID, &str> = iproduct!(0..coords.rows(), 0..coords.columns())
            .map(|(row, column)| players(&Coord { x: column, y: row }))
            .filter_map(|player: Option<Player<'a>>| player.map(|player| (player.id, player.name)))
            .collect();
        let mut texts: Vec<(PlayerID, &str)> = players.into_iter().collect();
        texts.push((DUMMY_PLAYER_ID, &mines_text));
        texts.push((DUMMY_PLAYER_ID, &lives_text));
        texts.sort_by_key(|&(id, _name)| id);
        texts
    };

    draw_text(
        texts.iter().map(|(id, name)| (*id, (*name).to_string())),
        texts.len(),
        context,
        graphics,
        canvas,
        canvas_size,
        fit,
        glyphs,
        v,
    )
}

#[allow(clippy::too_many_arguments)]
fn draw_text<G, T>(
    texts: T,
    texts_count: usize,
    context: &pw::Context,
    graphics: &mut G,
    canvas: &Canvas,
    canvas_size: &CanvasSize,
    fit: Option<Fit>,
    glyphs: &mut pw::Glyphs,
    v: &rusttype::VMetrics,
) where
    G: pw::Graphics<Texture = GLTexture>,
    T: Iterator<Item = (PlayerID, String)>,
{
    let font_size = 100;
    let scalar = 1. / font_size as f64;
    let pinv = 1.0 / texts_count as f64;
    let h = pinv * canvas.h;
    let scalar_v = v.ascent / (v.ascent - v.descent);
    for (y, (id, txt)) in texts.enumerate() {
        let canvas = Canvas {
            x: canvas.x,
            w: canvas.w,
            y: canvas.y + y as f64 * h,
            h,
        };
        let canvas = fit.map_or(canvas.clone(), |fit| {
            fit.rebuild_canvas(
                &canvas,
                canvas_size,
                CanvasRatio {
                    w_over_h: txt.len() as f64,
                },
            )
        });

        let transform = rect_transform(&context.transform, &canvas, canvas_size);
        let rectangle = pw::Rectangle::new(id.background_color());
        let dims = [0., 0., 1., 1.];
        rectangle.draw(dims, &context.draw_state, transform, graphics);

        let ninv = 1.0 / txt.len() as f64;
        let transform = transform
            .scale(ninv, 1.)
            .scale(1.0, scalar_v as f64)
            .trans(0.0, 1.0)
            .scale(scalar, scalar);

        pw::text::Text::new_color(id.color(), font_size)
            .draw(&txt, glyphs, &context.draw_state, transform, graphics)
            .expect("Failed to render text");
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_statistics_text<G>(
    context: &pw::Context,
    graphics: &mut G,
    canvas: &Canvas,
    canvas_size: &CanvasSize,
    fit: Option<Fit>,
    glyphs: &mut pw::Glyphs,
    v: &rusttype::VMetrics,
    stats: &status::Statistics,
    success: bool,
) where
    G: pw::Graphics<Texture = GLTexture>,
{
    let single = |descr: &str| (DUMMY_PLAYER_ID, descr.to_string());
    let convert = |stat: &status::UserStat| {
        let status::UserStat { id, name, number } = stat;
        (*id, format!("{} ({})", name, number))
    };
    let summary = if success { "YOU WON!" } else { "You Failed" };
    let texts = std::iter::once(single(summary))
        .chain(std::iter::once(single("Correct Flags:")))
        .chain(stats.marked_correct.iter().map(convert))
        .chain(std::iter::once(single("Exploded Mines:")))
        .chain(stats.exploded.iter().map(convert))
        .chain(std::iter::once(single("InCorrect Flags:")))
        .chain(stats.marked_incorrect.iter().map(convert));

    draw_text(
        texts,
        4 + stats.marked_correct.len() + stats.exploded.len() + stats.marked_incorrect.len(),
        context,
        graphics,
        canvas,
        canvas_size,
        fit,
        glyphs,
        v,
    )
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PlayerID(pub u8);

#[derive(Copy, Clone)]
pub struct Player<'a> {
    pub id: PlayerID,
    pub name: &'a str,
}

impl PlayerID {
    fn color(&self) -> Color {
        let &Self(id) = self;
        let index = id as usize;
        let colors: [Color; 1 + MAX_PLAYERS as usize] = [
            BLACK,
            BLUE,
            GREEN,
            YELLOW,
            PURPLE,
            ORANGE,
            BROWN,
            GRAY,
            CYAN,
            LIGHT_GREEN,
            MAGENTA,
        ];
        *colors.iter().nth(index).unwrap_or_else(|| {
            panic!("maximum number of player colors is 1+{MAX_PLAYERS}, player index = {index}")
        })
    }

    fn background_color(&self) -> Color {
        let &Self(id) = self;
        let index = id as usize;
        let colors: [Color; 1 + MAX_PLAYERS as usize] = [
            WHITE, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK,
        ];
        *colors.iter().nth(index).unwrap_or_else(|| {
            panic!("maximum number of player colors is 1+{MAX_PLAYERS}, player index = {index}")
        })
    }
}

pub fn run_window<S>(title: &str, mut status_generator: S)
where
    S: status::StatusGenerator,
{
    let font: rusttype::Font<'static> = rusttype::Font::try_from_bytes(FONT_DATA)
        .unwrap_or_else(|| panic!("Unable to construct font"));
    let v_metrics = font.v_metrics_unscaled();

    let mut window: pw::PistonWindow =
        pw::WindowSettings::new(title, (WINDOW_DEFAULT_WIDTH, WINDOW_DEFAULT_HEIGHT))
            .exit_on_esc(true)
            .build()
            .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));

    let mut glyphs = pw::Glyphs::from_font(
        font,
        window.create_texture_context(),
        pw::TextureSettings::new(),
    );
    let textures = &textures::Textures::new(window.create_texture_context());

    let mut input = input::Input::default();
    while let Some(event) = window.next() {
        status_generator.status(|status| match status {
            status::Status::Active(active) => active_event(
                active,
                event,
                &mut input,
                &mut window,
                textures,
                &mut glyphs,
                &v_metrics,
            ),
            status::Status::NonActive { nonactive, success } => nonactive_event(
                nonactive,
                event,
                &mut input,
                &mut window,
                &mut glyphs,
                &v_metrics,
                success,
            ),
        })
    }
}

enum CheckInput {
    MouseLeft,
    MouseRight,
}

fn check_input(i: pw::Input, input: &mut input::Input) -> Option<CheckInput> {
    let mut ci = None;
    match i {
        pw::Input::Resize(pw::ResizeArgs {
            draw_size: d,
            window_size: _,
        }) => {
            input.draw_size = d.into();
        }
        pw::Input::Move(pw::Motion::MouseCursor(cursor)) => {
            input.cursor_pos = cursor.into();
        }
        pw::Input::Button(pw::ButtonArgs {
            state,
            button: pw::Button::Mouse(pw::MouseButton::Right),
            scancode: _,
        }) => input.mouse_down.right(state, || {
            ci.replace(CheckInput::MouseRight);
        }),
        pw::Input::Button(pw::ButtonArgs {
            state,
            button: pw::Button::Mouse(pw::MouseButton::Left),
            scancode: _,
        }) => input.mouse_down.left(state, || {
            ci.replace(CheckInput::MouseLeft);
        }),
        _ => {}
    }
    ci
}

fn active_event<G>(
    mut active: status::Active<G>,
    event: pw::Event,
    input: &mut input::Input,
    window: &mut pw::PistonWindow,
    textures: &textures::Textures<GLResources>,
    glyphs: &mut pw::Glyphs,
    v_metrics: &rusttype::VMetrics,
) where
    G: status::Grid,
{
    // fixed dimensions
    let grid_canvas = Canvas {
        x: 0.02,
        y: 0.02,
        w: 0.76,
        h: 0.96,
    };
    let names_canvas = Canvas {
        x: 0.82,
        y: 0.02,
        w: 0.16,
        h: 0.96,
    };
    let fit = Some(Fit(FitHorizontal::Center, FitVertical::Center));

    let cursor_pos = input.cursor_pos;
    let draw_size = input.draw_size;
    let cell_clicked = || {
        identify_cell(
            &cursor_pos,
            &draw_size,
            &pw::math::identity(),
            &active.coords,
            &grid_canvas,
            fit,
        )
    };
    match event {
        pw::Event::Input(i, _) => match check_input(i, input) {
            Some(CheckInput::MouseRight) => {
                if let Some(coord) = cell_clicked() {
                    active.grid.right_click_cell(&coord);
                }
            }
            Some(CheckInput::MouseLeft) => {
                if let Some(coord) = cell_clicked() {
                    active.grid.left_click_cell(&coord);
                }
            }
            None => {}
        },
        pw::Event::Loop(pw::Loop::Render(render_args)) => {
            let grid = status::GridWithTextures::new(&active.grid, textures);
            window.draw_2d(&event, |c, g, d| {
                pw::clear(BACKGROUND_COLOR, g);
                draw_grid(
                    &c,
                    g,
                    &active.coords,
                    &grid_canvas,
                    &render_args.draw_size.into(),
                    fit,
                    |coord| grid.get_texture_and_player(coord),
                );
                draw_active_text(
                    &c,
                    g,
                    &active.coords,
                    &names_canvas,
                    &render_args.draw_size.into(),
                    fit,
                    glyphs,
                    v_metrics,
                    |coord| active.grid.get_cell(coord).player().copied(),
                    active.lives_left,
                    active.mines_left,
                );
                glyphs.factory.encoder.flush(d);
            });
        }
        _ => {}
    }
}

fn nonactive_event<C>(
    nonactive: status::NonActive<C>,
    event: pw::Event,
    input: &mut input::Input,
    window: &mut pw::PistonWindow,
    glyphs: &mut pw::Glyphs,
    v_metrics: &rusttype::VMetrics,
    success: bool,
) where
    C: status::Controller,
{
    let status::NonActive {
        stats: statistics,
        mut controller,
    } = nonactive;

    // fixed dimensions
    let canvas = Canvas {
        x: 0.02,
        y: 0.02,
        w: 0.96,
        h: 0.96,
    };
    let fit = Some(Fit(FitHorizontal::Center, FitVertical::Center));

    match event {
        pw::Event::Loop(pw::Loop::Render(render_args)) => {
            window.draw_2d(&event, |c, g, d| {
                pw::clear(BACKGROUND_COLOR, g);
                draw_statistics_text(
                    &c,
                    g,
                    &canvas,
                    &render_args.draw_size.into(),
                    fit,
                    glyphs,
                    v_metrics,
                    statistics,
                    success,
                );
                glyphs.factory.encoder.flush(d);
            });
        }
        pw::Event::Input(i, _) => match i {
            pw::Input::Button(pw::ButtonArgs {
                state: pw::ButtonState::Press,
                button: pw::Button::Keyboard(pw::Key::Space),
                scancode: _,
            }) => {
                controller.request_new_game();
            }
            _ => {
                check_input(i, input);
            }
        },
        _ => {}
    }
}
