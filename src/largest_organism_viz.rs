// Visualizer for displaying the largest organism in the simulation

use crate::grid::{Cell, CellType, Organism};
use iced::widget::canvas::{self, Cache, Canvas, Frame, Geometry, Path};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};

pub struct LargestOrganismViz {
    organism: Option<Organism>,
    cache: Cache,
}

impl LargestOrganismViz {
    pub fn new() -> Self {
        Self {
            organism: None,
            cache: Cache::default(),
        }
    }

    pub fn update(&mut self, organism: Option<Organism>) {
        self.organism = organism;
        self.cache.clear();
    }

    pub fn view(&self) -> Element<'_, ()> {
        Canvas::new(self)
            .width(Length::Fixed(200.0))
            .height(Length::Fixed(200.0))
            .into()
    }

    fn get_color_for_cell_type(cell_type: CellType) -> Color {
        match cell_type {
            CellType::Empty => Color::from_rgb8(0x30, 0x34, 0x3B),
            CellType::Alive => Color::from_rgb8(255, 100, 150), // Pink
            CellType::Food => Color::from_rgb8(255, 200, 100),  // Orange
            CellType::Grower => Color::from_rgb8(100, 255, 150), // Green
            CellType::Mover => Color::from_rgb8(100, 200, 255), // Cyan
        }
    }
}

impl canvas::Program<()> for LargestOrganismViz {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            // Background
            let background = Path::rectangle(Point::ORIGIN, frame.size());
            frame.fill(&background, Color::from_rgb8(0x1e, 0x1e, 0x2e));

            // Title
            let title = canvas::Text {
                content: "Largest Organism".to_string(),
                position: Point::new(10.0, 5.0),
                color: Color::from_rgba8(255, 255, 255, 0.7),
                size: 10.0.into(),
                ..canvas::Text::default()
            };
            frame.fill_text(title);

            if let Some(ref organism) = self.organism {
                if organism.cells.is_empty() {
                    let no_data_text = canvas::Text {
                        content: "No organism".to_string(),
                        position: Point::new(frame.width() / 2.0 - 40.0, frame.height() / 2.0),
                        color: Color::from_rgba8(255, 255, 255, 0.3),
                        size: 12.0.into(),
                        ..canvas::Text::default()
                    };
                    frame.fill_text(no_data_text);
                    return;
                }

                // Find bounds of the organism
                let min_i = organism.cells.iter().map(|c| c.i).min().unwrap_or(0);
                let max_i = organism.cells.iter().map(|c| c.i).max().unwrap_or(0);
                let min_j = organism.cells.iter().map(|c| c.j).min().unwrap_or(0);
                let max_j = organism.cells.iter().map(|c| c.j).max().unwrap_or(0);

                let org_width = (max_j - min_j + 1) as f32;
                let org_height = (max_i - min_i + 1) as f32;

                // Calculate cell size to fit in the frame (with padding)
                let padding = 30.0;
                let available_width = frame.width() - padding * 2.0;
                let available_height = frame.height() - padding * 2.0;

                let cell_size = (available_width / org_width)
                    .min(available_height / org_height)
                    .min(20.0); // Cap at reasonable size

                // Center the organism
                let offset_x = (frame.width() - org_width * cell_size) / 2.0;
                let offset_y = (frame.height() - org_height * cell_size) / 2.0 + 10.0;

                // Draw each cell
                for cell in &organism.cells {
                    let x = ((cell.j - min_j) as f32 * cell_size) + offset_x;
                    let y = ((cell.i - min_i) as f32 * cell_size) + offset_y;

                    let cell_rect = Path::rectangle(
                        Point::new(x, y),
                        iced::Size::new(cell_size - 1.0, cell_size - 1.0),
                    );

                    let color = Self::get_color_for_cell_type(cell.cell_type);
                    frame.fill(&cell_rect, color);

                    // Add a subtle border
                    frame.stroke(
                        &cell_rect,
                        canvas::Stroke::default()
                            .with_width(0.5)
                            .with_color(Color::from_rgba8(255, 255, 255, 0.2)),
                    );
                }

                // Display stats
                // println!("organism cells {:?}", organism.cells);
                let stats_text = canvas::Text {
                    content: format!(
                        "Cells: {} | Energy: {}",
                        organism.cells.len(),
                        organism.energy
                    ),
                    position: Point::new(10.0, frame.height() - 15.0),
                    color: Color::from_rgba8(255, 255, 255, 0.9),
                    size: 10.0.into(),
                    ..canvas::Text::default()
                };
                frame.fill_text(stats_text);

                // Movement indicator
                if organism.able_to_move {
                    let move_indicator = canvas::Text {
                        content: "Can move".to_string(),
                        position: Point::new(frame.width() - 25.0, 5.0),
                        color: Color::from_rgb8(255, 255, 100),
                        size: 14.0.into(),
                        ..canvas::Text::default()
                    };
                    frame.fill_text(move_indicator);
                }
            } else {
                let no_data_text = canvas::Text {
                    content: "No organisms yet".to_string(),
                    position: Point::new(frame.width() / 2.0 - 50.0, frame.height() / 2.0),
                    color: Color::from_rgba8(255, 255, 255, 0.3),
                    size: 12.0.into(),
                    ..canvas::Text::default()
                };
                frame.fill_text(no_data_text);
            }
        });

        vec![geometry]
    }
}
