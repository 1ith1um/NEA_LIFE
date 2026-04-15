// graph.rs
use iced::widget::canvas::{self, Cache, Canvas, Frame, Geometry, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};

pub struct PopulationGraph {
    data: Vec<usize>,
    cache: Cache,
}

impl PopulationGraph {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            cache: Cache::default(),
        }
    }

    pub fn update(&mut self, count: usize) {
        self.data.push(count);
        if self.data.len() > 1000 {
            self.data.remove(0);
        }
        self.cache.clear();
    }

    pub fn view(&self) -> Element<'_, ()> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fixed(150.0))
            .into()
    }
}

impl canvas::Program<()> for PopulationGraph {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        if self.data.is_empty() {
            return vec![];
        }

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            // Draw background
            let background = Path::rectangle(Point::ORIGIN, frame.size());
            frame.fill(&background, Color::from_rgb8(0x30, 0x34, 0x3B));

            // Calculate dimensions
            let max_value = *self.data.iter().max().unwrap_or(&1) as f32;
            let min_value = *self.data.iter().min().unwrap_or(&0) as f32;
            let value_range = (max_value - min_value).max(1.0);
            let width = frame.width();
            let height = frame.height();
            let step = width / self.data.len().max(1) as f32;

            // Build path without builder
            let mut points = Vec::new();
            for (i, &value) in self.data.iter().enumerate() {
                let x = i as f32 * step;
                let y = height - ((value as f32 - min_value) / value_range) * height;
                points.push(Point::new(x, y));
            }

            // Create path from points
            if !points.is_empty() {
                let path = Path::new(|p| {
                    p.move_to(points[0]);
                    for point in points.iter().skip(1) {
                        p.line_to(*point);
                    }
                });

                frame.stroke(
                    &path,
                    Stroke::default()
                        .with_width(2.0)
                        .with_color(Color::from_rgb(0.0, 0.8, 0.4)),
                );
            }

            // Add text label
            let text = canvas::Text {
                content: format!("Organisms: {}", self.data.last().unwrap_or(&0)),
                position: Point::new(10.0, 10.0),
                color: Color::WHITE,
                size: 12.0.into(),
                ..canvas::Text::default()
            };
            frame.fill_text(text);
        });

        vec![geometry]
    }
}
