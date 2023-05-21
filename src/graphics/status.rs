use super::*;

pub enum Cell<'a> {
    ExplodedMine(Player<'a>),
    Incorrect(Player<'a>),
    Flag(Player<'a>),
    Maybe(Player<'a>),
    Covered,
    Mine,
    EmptyNone,
    EmptyOne,
    EmptyTwo,
    EmptyThree,
    EmptyFour,
    EmptyFive,
    EmptySix,
    EmptySeven,
    EmptyEight,
}

impl<'a> Cell<'a> {
    pub fn player(&self) -> Option<&Player<'a>> {
        match self {
            Self::ExplodedMine(player) => Some(player),
            Self::Incorrect(player) => Some(player),
            Self::Flag(player) => Some(player),
            Self::Maybe(player) => Some(player),
            Self::Covered => None,
            Self::Mine => None,
            Self::EmptyNone => None,
            Self::EmptyOne => None,
            Self::EmptyTwo => None,
            Self::EmptyThree => None,
            Self::EmptyFour => None,
            Self::EmptyFive => None,
            Self::EmptySix => None,
            Self::EmptySeven => None,
            Self::EmptyEight => None,
        }
    }
}

pub trait Grid {
    fn get_cell<'a>(&'a self, coord: &Coord) -> Cell<'a>;
    fn left_click_cell(&mut self, coord: &Coord);
    fn right_click_cell(&mut self, coord: &Coord);
}

#[derive(Debug)]
pub struct LivesLeft(pub u32);

#[derive(Debug)]
pub struct MinesLeft(pub i32);

pub trait StatusGenerator {
    type Grid<'a>: Grid + 'a;
    type Controller<'a>: Controller + 'a;

    fn status<F, R>(&mut self, f: F) -> R
    where
        for<'a> F: FnOnce(Status<'a, Self::Controller<'a>, Self::Grid<'a>>) -> R;
}

pub enum Status<'a, C, G>
where
    C: Controller,
    G: Grid,
{
    Active(Active<G>),
    NonActive {
        nonactive: NonActive<'a, C>,
        success: bool,
    },
}

pub struct Active<G>
where
    G: Grid,
{
    pub coords: Coordinations,
    pub grid: G,
    pub lives_left: LivesLeft,
    pub mines_left: MinesLeft,
}

#[derive(Debug)]
pub struct NonActive<'a, C>
where
    C: Controller,
{
    pub stats: &'a Statistics,
    pub controller: C,
}

pub trait Controller {
    fn request_new_game(&mut self);
}

#[derive(Debug)]
pub struct Statistics {
    pub marked_correct: Vec<UserStat>,
    pub marked_incorrect: Vec<UserStat>,
    pub exploded: Vec<UserStat>,
}

#[derive(Debug)]
pub struct UserStat {
    pub id: PlayerID,
    pub name: String,
    pub number: u32,
}

pub struct GridWithTextures<'a, R>
where
    R: gfx::Resources,
{
    grid: &'a dyn status::Grid,
    textures: &'a textures::Textures<R>,
}

impl<R> GridWithTextures<'_, R>
where
    R: gfx::Resources,
{
    pub fn new<'a>(
        grid: &'a dyn status::Grid,
        textures: &'a textures::Textures<R>,
    ) -> GridWithTextures<'a, R> {
        GridWithTextures { grid, textures }
    }

    pub fn get_texture_and_player(
        &self,
        coord: &Coord,
    ) -> (Option<&pw::Texture<R>>, Option<PlayerID>) {
        let cell = self.grid.get_cell(coord);
        let player = cell.player().map(|player| player.id);
        let texture = match cell {
            Cell::Covered => Some(&self.textures.covered),
            Cell::Mine => Some(&self.textures.mine),
            Cell::ExplodedMine(_) => Some(&self.textures.exploded),
            Cell::Incorrect(_) => Some(&self.textures.incorrect),
            Cell::Flag(_) => Some(&self.textures.flag),
            Cell::Maybe(_) => Some(&self.textures.maybe),
            Cell::EmptyNone => None,
            Cell::EmptyOne => Some(&self.textures.one_mines),
            Cell::EmptyTwo => Some(&self.textures.two_mines),
            Cell::EmptyThree => Some(&self.textures.three_mines),
            Cell::EmptyFour => Some(&self.textures.four_mines),
            Cell::EmptyFive => Some(&self.textures.five_mines),
            Cell::EmptySix => Some(&self.textures.six_mines),
            Cell::EmptySeven => Some(&self.textures.seven_mines),
            Cell::EmptyEight => Some(&self.textures.eight_mines),
        };
        (texture, player)
    }
}
