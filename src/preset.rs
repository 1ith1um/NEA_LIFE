#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Preset {
    Custom,
    #[default]
    Blinker,
    Toad,
    Beacon,
    Pulsar,
    Pentadecathlon,
    Diehard,
    RPentomino,
    PiHeptomino,
    QueenBeeShuttle,
    SpaceRake,
}

pub static ALL: &[Preset] = &[
    Preset::Custom,
    Preset::Blinker,
    Preset::Toad,
    Preset::Beacon,
    Preset::Pulsar,
    Preset::Pentadecathlon,
    Preset::Diehard,
    Preset::RPentomino,
    Preset::PiHeptomino,
    Preset::QueenBeeShuttle,
    Preset::SpaceRake,
];

impl Preset {
    pub fn life(self) -> Vec<(isize, isize)> {
        // skip formatting
        #[rustfmt::skip]
        let cells = match self {
            // Preset cells are
            Preset::Custom => vec![],
            Preset::Blinker => vec![
                "     ",
                " xxx ",
                "     ",
            ],
            Preset::Toad => vec![
                "      ",
                "  xxx ",
                " xxx  ",
                "      ",
            ],
            Preset::Beacon => vec![
                "xx  ",
                "xx  ",
                "  xx",
                "  xx",
            ],
            Preset::Pulsar => vec![
                "  xxx   xxx  ",
                "             ",
                "x    x x    x",
                "x    x x    x",
                "x    x x    x",
                "  xxx   xxx  ",
                "             ",
                "  xxx   xxx  ",
                "x    x x    x",
                "x    x x    x",
                "x    x x    x",
                "             ",
                "  xxx   xxx  ",
            ],
            Preset::Pentadecathlon => vec![
                "    x    ",
                "    x    ",
                "xxxxxxxx",
                "    x    ",
                "    x    ",
            ],
            Preset::Diehard => vec![
                "       x",
                "xx     ",
                " x   xxx",
            ],
            Preset::RPentomino => vec![
                " xx",
                "xx ",
                " x ",
            ],
            Preset::PiHeptomino => vec![
                " xx",
                "xx ",
                " x ",
                "   x",
            ],
            Preset::QueenBeeShuttle => vec![
                "     x     ",
                "    x x    ",
                "   x   x   ",
                "  x     x  ",
                " x       x ",
                "x         x",
                " x       x ",
                "  x     x  ",
                "   x   x   ",
                "    x x    ",
                "     x     ",
            ],
            Preset::SpaceRake => vec![
                "     x",
                "    x ",
                "   x x",
                "   x x   x",
                "         x",
                "        x ",
            ],
        };

        let start_row = -(cells.len() as isize / 2);

        cells
            .into_iter()
            .enumerate()
            .flat_map(|(i, row)| {
                let start_column = -(row.len() as isize / 2);
                row.chars()
                    .enumerate()
                    .filter(|&(_, c)| c == 'x')
                    .map(move |(j, _)| (start_row + i as isize, start_column + j as isize))
            })
            .collect()
    }
}

impl std::fmt::Display for Preset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Preset::Custom => "Custom",
                Preset::Blinker => "Blinker",
                Preset::Toad => "Toad",
                Preset::Beacon => "Beacon",
                Preset::Pulsar => "Pulsar",
                Preset::Pentadecathlon => "Pentadecathlon",
                Preset::Diehard => "Diehard",
                Preset::RPentomino => "R-pentomino",
                Preset::PiHeptomino => "Pi-heptomino",
                Preset::QueenBeeShuttle => "Queen Bee Shuttle",
                Preset::SpaceRake => "Space Rake",
            }
        )
    }
}
