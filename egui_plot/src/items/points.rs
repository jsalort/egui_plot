use std::ops::RangeInclusive;

use egui::Color32;
use egui::Id;
use egui::Shape;
use egui::Stroke;
use egui::Ui;
use egui::epaint::CircleShape;
use emath::Pos2;
use emath::pos2;
use emath::vec2;

use crate::aesthetics::LineStyle;
use crate::aesthetics::MarkerShape;
use crate::axis::PlotTransform;
use crate::bounds::PlotBounds;
use crate::bounds::PlotPoint;
use crate::data::PlotPoints;
use crate::items::PlotGeometry;
use crate::items::PlotItem;
use crate::items::PlotItemBase;

impl<'a> Points<'a> {
    pub fn new(name: impl Into<String>, series: impl Into<PlotPoints<'a>>) -> Self {
        Self {
            base: PlotItemBase::new(name.into()),
            series: series.into(),
            shape: MarkerShape::Circle,
            color: Color32::TRANSPARENT,
            filled: true,
            radius: 1.0,
            line: false,
            stroke: Stroke::new(1.5, Color32::TRANSPARENT), // Note: a stroke of 1.0 (or less) can look bad on low-dpi-screens
            style: LineStyle::Solid,
            stems: None,
        }
    }

    /// Set the shape of the markers.
    #[inline]
    pub fn shape(mut self, shape: MarkerShape) -> Self {
        self.shape = shape;
        self
    }

    /// Set the both marker and line 's color.
    #[inline]
    pub fn color(mut self, color: impl Into<Color32>) -> Self {
        self.color = color.into();
        self.stroke.color = self.color.clone();
        self
    }

    /// Set the marker's color.
    #[inline]
    pub fn marker_color(mut self, color: impl Into<Color32>) -> Self {
        self.color = color.into();
        self
    }

    /// Set the line's color.
    #[inline]
    pub fn line_color(mut self, color: impl Into<Color32>) -> Self {
        self.stroke.color = color.into();
        self
    }

    /// Toggle line between points
    #[inline]
    pub fn line(mut self, shown: bool) -> Self {
        self.line = shown;
        self
    }

    /// Add a stroke to the line
    #[inline]
    pub fn line_stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = stroke.into();
        self
    }

    /// Stroke width. A high value means the plot thickens.
    #[inline]
    pub fn line_width(mut self, width: impl Into<f32>) -> Self {
        self.stroke.width = width.into();
        self
    }

    /// Set the line's style. Default is `LineStyle::Solid`.
    #[inline]
    pub fn line_style(mut self, style: LineStyle) -> Self {
        self.style = style;
        self
    }

    /// Whether to fill the marker.
    #[inline]
    pub fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }

    /// Whether to add stems between the markers and a horizontal reference
    /// line.
    #[inline]
    pub fn stems(mut self, y_reference: impl Into<f32>) -> Self {
        self.stems = Some(y_reference.into());
        self
    }

    /// Set the maximum extent of the marker around its position, in ui points.
    #[inline]
    pub fn radius(mut self, radius: impl Into<f32>) -> Self {
        self.radius = radius.into();
        self
    }

    /// Name of this plot item.
    ///
    /// This name will show up in the plot legend, if legends are turned on.
    ///
    /// Setting the name via this method does not change the item's id, so you
    /// can use it to change the name dynamically between frames without
    /// losing the item's state. You should make sure the name passed to
    /// [`Self::new`] is unique and stable for each item, or set unique and
    /// stable ids explicitly via [`Self::id`].
    #[expect(clippy::needless_pass_by_value, reason = "to allow various string types")]
    #[inline]
    pub fn name(mut self, name: impl ToString) -> Self {
        self.base_mut().name = name.to_string();
        self
    }

    /// Highlight this plot item, typically by scaling it up.
    ///
    /// If false, the item may still be highlighted via user interaction.
    #[inline]
    pub fn highlight(mut self, highlight: bool) -> Self {
        self.base_mut().highlight = highlight;
        self
    }

    /// Allowed hovering this item in the plot. Default: `true`.
    #[inline]
    pub fn allow_hover(mut self, hovering: bool) -> Self {
        self.base_mut().allow_hover = hovering;
        self
    }

    /// Sets the id of this plot item.
    ///
    /// By default the id is determined from the name passed to [`Self::new`],
    /// but it can be explicitly set to a different value.
    #[inline]
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.base_mut().id = id.into();
        self
    }
}

/// A set of points.
pub struct Points<'a> {
    base: PlotItemBase,

    pub(crate) series: PlotPoints<'a>,

    pub(crate) shape: MarkerShape,

    /// Color of the marker. `Color32::TRANSPARENT` means that it will be picked
    /// automatically.
    pub(crate) color: Color32,

    /// Whether to fill the marker. Does not apply to all types.
    pub(crate) filled: bool,

    /// The maximum extent of the marker from its center.
    pub(crate) radius: f32,

    /// Draw line between points
    pub(super) line: bool,

    /// Color of the line (if drawn)
    pub(super) stroke: Stroke,

    /// Style of the line (if drawn)
    pub(super) style: LineStyle,

    pub(crate) stems: Option<f32>,
}

impl PlotItem for Points<'_> {
    fn shapes(&self, _ui: &Ui, transform: &PlotTransform, shapes: &mut Vec<Shape>) {
        let sqrt_3 = 3_f32.sqrt();
        let frac_sqrt_3_2 = 3_f32.sqrt() / 2.0;
        let frac_1_sqrt_2 = 1.0 / 2_f32.sqrt();

        let Self {
            base,
            series,
            shape,
            color,
            filled,
            radius,
            line,
            stroke,
            style,
            stems,
            ..
        } = self;

        let mut radius = *radius;

        let line_stroke = *stroke;
        let stroke_size = radius / 5.0;

        let default_stroke = Stroke::new(stroke_size, *color);
        let mut stem_stroke = default_stroke;
        let (fill, stroke) = if *filled {
            (*color, Stroke::NONE)
        } else {
            (Color32::TRANSPARENT, default_stroke)
        };

        if base.highlight {
            radius *= 2f32.sqrt();
            stem_stroke.width *= 2.0;
        }

        let y_reference = stems.map(|y| transform.position_from_point(&PlotPoint::new(0.0, y)).y);

        series
            .points()
            .iter()
            .map(|value| transform.position_from_point(value))
            .for_each(|center| {
                let tf = |dx: f32, dy: f32| -> Pos2 { center + radius * vec2(dx, dy) };

                if let Some(y) = y_reference {
                    let stem = Shape::line_segment([center, pos2(center.x, y)], stem_stroke);
                    shapes.push(stem);
                }

                match shape {
                    MarkerShape::Circle => {
                        shapes.push(Shape::Circle(CircleShape {
                            center,
                            radius,
                            fill,
                            stroke,
                        }));
                    }
                    MarkerShape::Diamond => {
                        let points = vec![
                            tf(0.0, 1.0),  // bottom
                            tf(-1.0, 0.0), // left
                            tf(0.0, -1.0), // top
                            tf(1.0, 0.0),  // right
                        ];
                        shapes.push(Shape::convex_polygon(points, fill, stroke));
                    }
                    MarkerShape::Square => {
                        let points = vec![
                            tf(-frac_1_sqrt_2, frac_1_sqrt_2),
                            tf(-frac_1_sqrt_2, -frac_1_sqrt_2),
                            tf(frac_1_sqrt_2, -frac_1_sqrt_2),
                            tf(frac_1_sqrt_2, frac_1_sqrt_2),
                        ];
                        shapes.push(Shape::convex_polygon(points, fill, stroke));
                    }
                    MarkerShape::Cross => {
                        let diagonal1 = [tf(-frac_1_sqrt_2, -frac_1_sqrt_2), tf(frac_1_sqrt_2, frac_1_sqrt_2)];
                        let diagonal2 = [tf(frac_1_sqrt_2, -frac_1_sqrt_2), tf(-frac_1_sqrt_2, frac_1_sqrt_2)];
                        shapes.push(Shape::line_segment(diagonal1, default_stroke));
                        shapes.push(Shape::line_segment(diagonal2, default_stroke));
                    }
                    MarkerShape::Plus => {
                        let horizontal = [tf(-1.0, 0.0), tf(1.0, 0.0)];
                        let vertical = [tf(0.0, -1.0), tf(0.0, 1.0)];
                        shapes.push(Shape::line_segment(horizontal, default_stroke));
                        shapes.push(Shape::line_segment(vertical, default_stroke));
                    }
                    MarkerShape::Up => {
                        let points = vec![tf(0.0, -1.0), tf(0.5 * sqrt_3, 0.5), tf(-0.5 * sqrt_3, 0.5)];
                        shapes.push(Shape::convex_polygon(points, fill, stroke));
                    }
                    MarkerShape::Down => {
                        let points = vec![tf(0.0, 1.0), tf(-0.5 * sqrt_3, -0.5), tf(0.5 * sqrt_3, -0.5)];
                        shapes.push(Shape::convex_polygon(points, fill, stroke));
                    }
                    MarkerShape::Left => {
                        let points = vec![tf(-1.0, 0.0), tf(0.5, -0.5 * sqrt_3), tf(0.5, 0.5 * sqrt_3)];
                        shapes.push(Shape::convex_polygon(points, fill, stroke));
                    }
                    MarkerShape::Right => {
                        let points = vec![tf(1.0, 0.0), tf(-0.5, 0.5 * sqrt_3), tf(-0.5, -0.5 * sqrt_3)];
                        shapes.push(Shape::convex_polygon(points, fill, stroke));
                    }
                    MarkerShape::Asterisk => {
                        let vertical = [tf(0.0, -1.0), tf(0.0, 1.0)];
                        let diagonal1 = [tf(-frac_sqrt_3_2, 0.5), tf(frac_sqrt_3_2, -0.5)];
                        let diagonal2 = [tf(-frac_sqrt_3_2, -0.5), tf(frac_sqrt_3_2, 0.5)];
                        shapes.push(Shape::line_segment(vertical, default_stroke));
                        shapes.push(Shape::line_segment(diagonal1, default_stroke));
                        shapes.push(Shape::line_segment(diagonal2, default_stroke));
                    }
                }
            });
        if *line {
            let values_tf: Vec<_> = series
                .points()
                .iter()
                .map(|v| transform.position_from_point(v))
                .collect();
            style.style_line(values_tf, line_stroke.into(), base.highlight, shapes);
        }
    }

    fn initialize(&mut self, x_range: RangeInclusive<f64>) {
        self.series.generate_points(x_range);
    }

    fn color(&self) -> Color32 {
        self.color
    }

    fn geometry(&self) -> PlotGeometry<'_> {
        PlotGeometry::Points(self.series.points())
    }

    fn bounds(&self) -> PlotBounds {
        self.series.bounds()
    }

    fn base(&self) -> &PlotItemBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut PlotItemBase {
        &mut self.base
    }
}
