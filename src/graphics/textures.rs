use super::*;

static UNCOVERED_PNG: &[u8] = include_bytes!("png/covered.png");
static MINE_PNG: &[u8] = include_bytes!("png/mine.png");
static EXPLODED_PNG: &[u8] = include_bytes!("png/exploded.png");
static INCORRECT_PNG: &[u8] = include_bytes!("png/incorrect.png");
static FLAG_PNG: &[u8] = include_bytes!("png/flag.png");
static MAYBE_PNG: &[u8] = include_bytes!("png/maybe.png");
static ONE_MINES_PNG: &[u8] = include_bytes!("png/1mines.png");
static TWO_MINES_PNG: &[u8] = include_bytes!("png/2mines.png");
static THREE_MINES_PNG: &[u8] = include_bytes!("png/3mines.png");
static FOUR_MINES_PNG: &[u8] = include_bytes!("png/4mines.png");
static FIVE_MINES_PNG: &[u8] = include_bytes!("png/5mines.png");
static SIX_MINES_PNG: &[u8] = include_bytes!("png/6mines.png");
static SEVEN_MINES_PNG: &[u8] = include_bytes!("png/7mines.png");
static EIGHT_MINES_PNG: &[u8] = include_bytes!("png/8mines.png");

fn load_texture<F, R, C>(
    context: &mut pw::TextureContext<F, R, C>,
    png: &'static [u8],
) -> pw::Texture<R>
where
    F: gfx::Factory<R>,
    R: gfx::Resources,
    C: gfx_core::command::Buffer<R>,
{
    let settings = pw::TextureSettings::new();
    let buffer = image::load_from_memory_with_format(png, image::ImageFormat::Png)
        .expect("Failed to create imagebuffer");
    let buffer = buffer.as_rgba8().expect("Unexpected png format");
    pw::Texture::from_image(context, buffer, &settings).expect("Failed to generate texture")
}

pub struct Textures<R>
where
    R: gfx::Resources,
{
    pub mine: pw::Texture<R>,
    pub covered: pw::Texture<R>,
    pub exploded: pw::Texture<R>,
    pub incorrect: pw::Texture<R>,
    pub flag: pw::Texture<R>,
    pub maybe: pw::Texture<R>,
    pub one_mines: pw::Texture<R>,
    pub two_mines: pw::Texture<R>,
    pub three_mines: pw::Texture<R>,
    pub four_mines: pw::Texture<R>,
    pub five_mines: pw::Texture<R>,
    pub six_mines: pw::Texture<R>,
    pub seven_mines: pw::Texture<R>,
    pub eight_mines: pw::Texture<R>,
}

impl<R> Textures<R>
where
    R: gfx::Resources,
{
    pub fn new<F, C>(mut context: pw::TextureContext<F, R, C>) -> Self
    where
        F: gfx::Factory<R>,
        C: gfx_core::command::Buffer<R>,
    {
        Self {
            mine: load_texture(&mut context, MINE_PNG),
            covered: load_texture(&mut context, UNCOVERED_PNG),
            exploded: load_texture(&mut context, EXPLODED_PNG),
            incorrect: load_texture(&mut context, INCORRECT_PNG),
            flag: load_texture(&mut context, FLAG_PNG),
            maybe: load_texture(&mut context, MAYBE_PNG),
            one_mines: load_texture(&mut context, ONE_MINES_PNG),
            two_mines: load_texture(&mut context, TWO_MINES_PNG),
            three_mines: load_texture(&mut context, THREE_MINES_PNG),
            four_mines: load_texture(&mut context, FOUR_MINES_PNG),
            five_mines: load_texture(&mut context, FIVE_MINES_PNG),
            six_mines: load_texture(&mut context, SIX_MINES_PNG),
            seven_mines: load_texture(&mut context, SEVEN_MINES_PNG),
            eight_mines: load_texture(&mut context, EIGHT_MINES_PNG),
        }
    }
}
