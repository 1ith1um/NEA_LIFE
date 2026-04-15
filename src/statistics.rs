// statistics.rs
// Enhanced statistics tracking and visualization module

use iced::widget::canvas::{self, Cache, Canvas, Frame, Geometry, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};

// ═══════════════════════════════════════════════════════════════════════════
// Statistics Data Structure
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct SimulationStats {
    pub tick: usize,
    pub organism_count: usize,
    pub avg_organism_size: f32,
    pub largest_organism_size: usize,
    pub total_alive_cells: usize,
    pub total_grower_cells: usize,
    pub total_mover_cells: usize,
    pub total_food_cells: usize,
    pub avg_energy: f32,
    pub total_energy: usize,
}

impl Default for SimulationStats {
    fn default() -> Self {
        Self {
            tick: 0,
            organism_count: 0,
            avg_organism_size: 0.0,
            largest_organism_size: 0,
            total_alive_cells: 0,
            total_grower_cells: 0,
            total_mover_cells: 0,
            total_food_cells: 0,
            avg_energy: 0.0,
            total_energy: 0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Population Graph (Enhanced)
// ═══════════════════════════════════════════════════════════════════════════

pub struct PopulationGraph {
    data: Vec<usize>,
    cache: Cache,
    max_points: usize,
}

impl PopulationGraph {
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    pub fn with_capacity(max_points: usize) -> Self {
        Self {
            data: Vec::new(),
            cache: Cache::default(),
            max_points,
        }
    }

    pub fn update(&mut self, count: usize) {
        self.data.push(count);
        if self.data.len() > self.max_points {
            self.data.remove(0);
        }
        self.cache.clear();
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.cache.clear();
    }

    pub fn view(&self) -> Element<'_, ()> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fixed(120.0))
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
            // Background
            let background = Path::rectangle(Point::ORIGIN, frame.size());
            frame.fill(&background, Color::from_rgb8(0x1e, 0x1e, 0x2e));

            // Calculate dimensions
            let max_value = *self.data.iter().max().unwrap_or(&1) as f32;
            let min_value = *self.data.iter().min().unwrap_or(&0) as f32;
            let value_range = (max_value - min_value).max(1.0);
            let width = frame.width();
            let height = frame.height();
            let step = width / self.data.len().max(1) as f32;

            // Draw grid lines
            for i in 0..5 {
                let y = (height / 4.0) * i as f32;
                let grid_line = Path::line(Point::new(0.0, y), Point::new(width, y));
                frame.stroke(
                    &grid_line,
                    Stroke::default()
                        .with_width(1.0)
                        .with_color(Color::from_rgba8(255, 255, 255, 0.1)),
                );
            }

            // Build path
            let mut points = Vec::new();
            for (i, &value) in self.data.iter().enumerate() {
                let x = i as f32 * step;
                let y = height - ((value as f32 - min_value) / value_range) * height;
                points.push(Point::new(x, y));
            }

            // Draw filled area under the line
            if points.len() > 1 {
                let mut area_points = points.clone();
                area_points.push(Point::new(width, height));
                area_points.push(Point::new(0.0, height));

                let area_path = Path::new(|p| {
                    p.move_to(area_points[0]);
                    for point in area_points.iter().skip(1) {
                        p.line_to(*point);
                    }
                    p.close();
                });

                frame.fill(&area_path, Color::from_rgba8(0, 204, 102, 0.3));
            }

            // Draw line
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
                        .with_color(Color::from_rgb8(0, 204, 102)),
                );
            }

            // Labels
            let title = canvas::Text {
                content: "Population".to_string(),
                position: Point::new(10.0, 5.0),
                color: Color::from_rgba8(255, 255, 255, 0.7),
                size: 10.0.into(),
                ..canvas::Text::default()
            };
            frame.fill_text(title);

            let value_text = canvas::Text {
                content: format!("{}", self.data.last().unwrap_or(&0)),
                position: Point::new(10.0, 20.0),
                color: Color::WHITE,
                size: 16.0.into(),
                ..canvas::Text::default()
            };
            frame.fill_text(value_text);

            // Max value label
            let max_label = canvas::Text {
                content: format!("Peak: {}", max_value as usize),
                position: Point::new(width - 80.0, 5.0),
                color: Color::from_rgba8(255, 255, 255, 0.5),
                size: 10.0.into(),
                ..canvas::Text::default()
            };
            frame.fill_text(max_label);
        });

        vec![geometry]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Average Organism Size Graph
// ═══════════════════════════════════════════════════════════════════════════

pub struct AverageSizeGraph {
    data: Vec<f32>,
    cache: Cache,
    max_points: usize,
}

impl AverageSizeGraph {
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    pub fn with_capacity(max_points: usize) -> Self {
        Self {
            data: Vec::new(),
            cache: Cache::default(),
            max_points,
        }
    }

    pub fn update(&mut self, avg_size: f32) {
        self.data.push(avg_size);
        if self.data.len() > self.max_points {
            self.data.remove(0);
        }
        self.cache.clear();
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.cache.clear();
    }

    pub fn view(&self) -> Element<'_, ()> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fixed(120.0))
            .into()
    }
}

impl canvas::Program<()> for AverageSizeGraph {
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
            // Background
            let background = Path::rectangle(Point::ORIGIN, frame.size());
            frame.fill(&background, Color::from_rgb8(0x1e, 0x1e, 0x2e));

            let max_value = self.data.iter().fold(0.0f32, |a, &b| a.max(b));
            let min_value = self.data.iter().fold(f32::MAX, |a, &b| a.min(b));
            let value_range = (max_value - min_value).max(1.0);
            let width = frame.width();
            let height = frame.height();
            let step = width / self.data.len().max(1) as f32;

            // Draw grid
            for i in 0..5 {
                let y = (height / 4.0) * i as f32;
                let grid_line = Path::line(Point::new(0.0, y), Point::new(width, y));
                frame.stroke(
                    &grid_line,
                    Stroke::default()
                        .with_width(1.0)
                        .with_color(Color::from_rgba8(255, 255, 255, 0.1)),
                );
            }

            // Build points
            let mut points = Vec::new();
            for (i, &value) in self.data.iter().enumerate() {
                let x = i as f32 * step;
                let y = height - ((value - min_value) / value_range) * height;
                points.push(Point::new(x, y));
            }

            // Filled area
            if points.len() > 1 {
                let mut area_points = points.clone();
                area_points.push(Point::new(width, height));
                area_points.push(Point::new(0.0, height));

                let area_path = Path::new(|p| {
                    p.move_to(area_points[0]);
                    for point in area_points.iter().skip(1) {
                        p.line_to(*point);
                    }
                    p.close();
                });

                frame.fill(&area_path, Color::from_rgba8(102, 153, 255, 0.3));
            }

            // Line
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
                        .with_color(Color::from_rgb8(102, 153, 255)),
                );
            }

            // Labels
            let title = canvas::Text {
                content: "Avg Size".to_string(),
                position: Point::new(10.0, 5.0),
                color: Color::from_rgba8(255, 255, 255, 0.7),
                size: 10.0.into(),
                ..canvas::Text::default()
            };
            frame.fill_text(title);

            let value_text = canvas::Text {
                content: format!("{:.1}", self.data.last().unwrap_or(&0.0)),
                position: Point::new(10.0, 20.0),
                color: Color::WHITE,
                size: 16.0.into(),
                ..canvas::Text::default()
            };
            frame.fill_text(value_text);
        });

        vec![geometry]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Cell Type Distribution Graph (Stacked Area)
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Default)]
pub struct CellTypeDistribution {
    pub alive: usize,
    pub grower: usize,
    pub mover: usize,
    pub food: usize,
}

pub struct CellTypeGraph {
    data: Vec<CellTypeDistribution>,
    cache: Cache,
    max_points: usize,
}

impl CellTypeGraph {
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    pub fn with_capacity(max_points: usize) -> Self {
        Self {
            data: Vec::new(),
            cache: Cache::default(),
            max_points,
        }
    }

    pub fn update(&mut self, distribution: CellTypeDistribution) {
        self.data.push(distribution);
        if self.data.len() > self.max_points {
            self.data.remove(0);
        }
        self.cache.clear();
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.cache.clear();
    }

    pub fn view(&self) -> Element<'_, ()> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fixed(120.0))
            .into()
    }
}

impl canvas::Program<()> for CellTypeGraph {
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
            // Background
            let background = Path::rectangle(Point::ORIGIN, frame.size());
            frame.fill(&background, Color::from_rgb8(0x1e, 0x1e, 0x2e));

            let width = frame.width();
            let height = frame.height();
            let step = width / self.data.len().max(1) as f32;

            // Find max total
            let max_total = self
                .data
                .iter()
                .map(|d| d.alive + d.grower + d.mover + d.food)
                .max()
                .unwrap_or(1) as f32;

            // Draw stacked areas (from bottom to top: food, mover, grower, alive)
            let layers: [(Color, fn(&CellTypeDistribution) -> usize); 4] = [
                (
                    Color::from_rgb8(255, 200, 100),
                    |d: &CellTypeDistribution| d.food,
                ), // Food - orange
                (
                    Color::from_rgb8(100, 200, 255),
                    |d: &CellTypeDistribution| d.mover,
                ), // Mover - cyan
                (
                    Color::from_rgb8(100, 255, 150),
                    |d: &CellTypeDistribution| d.grower,
                ), // Grower - green
                (
                    Color::from_rgb8(255, 100, 150),
                    |d: &CellTypeDistribution| d.alive,
                ), // Alive - pink
            ];

            let mut cumulative_values = vec![0usize; self.data.len()];

            for (color, accessor) in layers.iter().rev() {
                let mut points = Vec::new();

                // Top edge of this layer
                for (i, dist) in self.data.iter().enumerate() {
                    let x = i as f32 * step;
                    let value = cumulative_values[i] + accessor(dist);
                    let y = height - (value as f32 / max_total) * height;
                    points.push(Point::new(x, y));
                    cumulative_values[i] = value;
                }

                // Bottom edge (previous layer's top)
                let mut area_points = points.clone();
                for i in (0..self.data.len()).rev() {
                    let x = i as f32 * step;
                    let prev_value = cumulative_values[i] - accessor(&self.data[i]);
                    let y = height - (prev_value as f32 / max_total) * height;
                    area_points.push(Point::new(x, y));
                }

                if !area_points.is_empty() {
                    let area_path = Path::new(|p| {
                        p.move_to(area_points[0]);
                        for point in area_points.iter().skip(1) {
                            p.line_to(*point);
                        }
                        p.close();
                    });
                    frame.fill(&area_path, *color);
                }
            }

            // Title
            let title = canvas::Text {
                content: "Cell Types".to_string(),
                position: Point::new(10.0, 5.0),
                color: Color::from_rgba8(255, 255, 255, 0.7),
                size: 10.0.into(),
                ..canvas::Text::default()
            };
            frame.fill_text(title);

            // Legend
            if let Some(last) = self.data.last() {
                let legend_y = 5.0;
                let legend_x_start = width - 150.0;

                let legend_items = [
                    ("A", Color::from_rgb8(255, 100, 150), last.alive),
                    ("G", Color::from_rgb8(100, 255, 150), last.grower),
                    ("M", Color::from_rgb8(100, 200, 255), last.mover),
                    ("F", Color::from_rgb8(255, 200, 100), last.food),
                ];

                for (i, (label, color, count)) in legend_items.iter().enumerate() {
                    let x = legend_x_start + (i as f32 * 37.0);

                    // Color box
                    let box_path =
                        Path::rectangle(Point::new(x, legend_y + 2.0), iced::Size::new(8.0, 8.0));
                    frame.fill(&box_path, *color);

                    // Label and count
                    let text = canvas::Text {
                        content: format!("{}:{}", label, count),
                        position: Point::new(x + 10.0, legend_y),
                        color: Color::from_rgba8(255, 255, 255, 0.8),
                        size: 9.0.into(),
                        ..canvas::Text::default()
                    };
                    frame.fill_text(text);
                }
            }
        });

        vec![geometry]
    }
}
