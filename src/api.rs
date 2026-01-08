use crate::error::Result;

/// Represents a color, gradient, or pattern that can be used for fill/stroke.
#[derive(Clone, Debug, PartialEq)]
pub enum Paint {
    Color(String),
    Gradient(CanvasGradient),
    Pattern(CanvasPattern),
}

#[derive(Clone, Debug, PartialEq)]
pub struct GradientStop {
    pub offset: f64,
    pub color: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GradientKind {
    Linear {
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
    },
    Radial {
        x0: f64,
        y0: f64,
        r0: f64,
        x1: f64,
        y1: f64,
        r1: f64,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanvasGradient {
    pub kind: GradientKind,
    pub stops: Vec<GradientStop>,
}

impl CanvasGradient {
    /// Mirrors CanvasGradient.addColorStop.
    pub fn add_color_stop(&mut self, offset: f64, color: impl Into<String>) {
        self.stops.push(GradientStop {
            offset,
            color: color.into(),
        });
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PatternRepetition {
    Repeat,
    RepeatX,
    RepeatY,
    NoRepeat,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanvasPattern {
    pub repetition: PatternRepetition,
    /// Optional 2D transform expressed as an SVG/Canvas DOMMatrix (a, b, c, d, e, f).
    pub transform: Option<[f64; 6]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextMetrics {
    pub width: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LineJoin {
    Round,
    Bevel,
    Miter,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Start,
    End,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TextBaseline {
    Top,
    Hanging,
    Middle,
    Alphabetic,
    Ideographic,
    Bottom,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Direction {
    Ltr,
    Rtl,
    Inherit,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CompositeOperation {
    SourceOver,
    SourceIn,
    SourceOut,
    SourceAtop,
    DestinationOver,
    DestinationIn,
    DestinationOut,
    DestinationAtop,
    Lighter,
    Copy,
    Xor,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HitOptions {
    pub fill_rule: FillRule,
}

impl Default for HitOptions {
    fn default() -> Self {
        Self {
            fill_rule: FillRule::NonZero,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImageSmoothingQuality {
    Low,
    Medium,
    High,
}

pub trait CanvasState {
    fn save(&mut self) -> Result<()>;
    fn restore(&mut self) -> Result<()>;
    fn reset(&mut self) -> Result<()>;

    fn set_global_alpha(&mut self, value: f64) -> Result<()>;
    fn global_alpha(&self) -> Result<f64>;

    fn set_global_composite_operation(&mut self, op: CompositeOperation) -> Result<()>;
    fn global_composite_operation(&self) -> Result<CompositeOperation>;

    fn set_image_smoothing_enabled(&mut self, enabled: bool) -> Result<()>;
    fn image_smoothing_enabled(&self) -> Result<bool>;

    fn set_image_smoothing_quality(&mut self, quality: ImageSmoothingQuality) -> Result<()>;
    fn image_smoothing_quality(&self) -> Result<ImageSmoothingQuality>;
}

pub trait CanvasTransforms {
    fn scale(&mut self, x: f64, y: f64) -> Result<()>;
    fn rotate(&mut self, radians: f64) -> Result<()>;
    fn translate(&mut self, x: f64, y: f64) -> Result<()>;
    fn transform(&mut self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Result<()>;
    fn set_transform(&mut self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Result<()>;
    fn reset_transform(&mut self) -> Result<()>;
}

pub trait CanvasCompositing {
    fn set_shadow_offset_x(&mut self, value: f64) -> Result<()>;
    fn shadow_offset_x(&self) -> Result<f64>;

    fn set_shadow_offset_y(&mut self, value: f64) -> Result<()>;
    fn shadow_offset_y(&self) -> Result<f64>;

    fn set_shadow_blur(&mut self, value: f64) -> Result<()>;
    fn shadow_blur(&self) -> Result<f64>;

    fn set_shadow_color(&mut self, value: String) -> Result<()>;
    fn shadow_color(&self) -> Result<String>;
}

pub trait CanvasLineStyles {
    fn set_line_width(&mut self, value: f64) -> Result<()>;
    fn line_width(&self) -> Result<f64>;

    fn set_line_cap(&mut self, value: LineCap) -> Result<()>;
    fn line_cap(&self) -> Result<LineCap>;

    fn set_line_join(&mut self, value: LineJoin) -> Result<()>;
    fn line_join(&self) -> Result<LineJoin>;

    fn set_miter_limit(&mut self, value: f64) -> Result<()>;
    fn miter_limit(&self) -> Result<f64>;

    fn set_line_dash(&mut self, segments: Vec<f64>) -> Result<()>;
    fn line_dash(&self) -> Result<Vec<f64>>;

    fn set_line_dash_offset(&mut self, value: f64) -> Result<()>;
    fn line_dash_offset(&self) -> Result<f64>;
}

pub trait CanvasFillStrokeStyles {
    fn set_fill_style(&mut self, style: Paint) -> Result<()>;
    fn fill_style(&self) -> Result<Paint>;

    fn set_stroke_style(&mut self, style: Paint) -> Result<()>;
    fn stroke_style(&self) -> Result<Paint>;

    fn create_linear_gradient(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) -> Result<CanvasGradient>;
    fn create_radial_gradient(
        &mut self,
        x0: f64,
        y0: f64,
        r0: f64,
        x1: f64,
        y1: f64,
        r1: f64,
    ) -> Result<CanvasGradient>;

    fn create_pattern(
        &mut self,
        image: &dyn CanvasImageSource,
        repetition: PatternRepetition,
    ) -> Result<CanvasPattern>;
}

pub trait CanvasRectangles {
    fn clear_rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> Result<()>;
    fn fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> Result<()>;
    fn stroke_rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> Result<()>;
}

pub trait CanvasPaths {
    fn begin_path(&mut self) -> Result<()>;
    fn close_path(&mut self) -> Result<()>;
    fn move_to(&mut self, x: f64, y: f64) -> Result<()>;
    fn line_to(&mut self, x: f64, y: f64) -> Result<()>;
    fn bezier_curve_to(&mut self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) -> Result<()>;
    fn quadratic_curve_to(&mut self, cpx: f64, cpy: f64, x: f64, y: f64) -> Result<()>;
    fn arc(&mut self, x: f64, y: f64, radius: f64, start_angle: f64, end_angle: f64, ccw: bool) -> Result<()>;
    fn arc_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, radius: f64) -> Result<()>;
    fn ellipse(
        &mut self,
        x: f64,
        y: f64,
        radius_x: f64,
        radius_y: f64,
        rotation: f64,
        start_angle: f64,
        end_angle: f64,
        ccw: bool,
    ) -> Result<()>;
    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> Result<()>;
    fn round_rect(&mut self, x: f64, y: f64, w: f64, h: f64, radii: &[f64]) -> Result<()>;

    fn fill(&mut self, fill_rule: FillRule) -> Result<()>;
    fn stroke(&mut self) -> Result<()>;
    fn clip(&mut self, fill_rule: FillRule) -> Result<()>;
    fn is_point_in_path(&self, x: f64, y: f64, opts: HitOptions) -> Result<bool>;
    fn is_point_in_stroke(&self, x: f64, y: f64) -> Result<bool>;
}

pub trait CanvasText {
    fn set_font(&mut self, value: String) -> Result<()>;
    fn font(&self) -> Result<String>;

    fn set_text_align(&mut self, value: TextAlign) -> Result<()>;
    fn text_align(&self) -> Result<TextAlign>;

    fn set_text_baseline(&mut self, value: TextBaseline) -> Result<()>;
    fn text_baseline(&self) -> Result<TextBaseline>;

    fn set_direction(&mut self, value: Direction) -> Result<()>;
    fn direction(&self) -> Result<Direction>;

    fn fill_text(&mut self, text: &str, x: f64, y: f64, max_width: Option<f64>) -> Result<()>;
    fn stroke_text(&mut self, text: &str, x: f64, y: f64, max_width: Option<f64>) -> Result<()>;
    fn measure_text(&self, text: &str) -> Result<TextMetrics>;
}

pub trait CanvasImageData {
    fn create_image_data(&mut self, width: u32, height: u32) -> Result<ImageData>;
    fn get_image_data(&self, sx: u32, sy: u32, sw: u32, sh: u32) -> Result<ImageData>;
    fn put_image_data(&mut self, data: &ImageData, dx: f64, dy: f64) -> Result<()>;
    fn put_image_data_dirty(
        &mut self,
        data: &ImageData,
        dx: f64,
        dy: f64,
        dirty_x: u32,
        dirty_y: u32,
        dirty_width: u32,
        dirty_height: u32,
    ) -> Result<()>;
}

pub trait CanvasDrawImage {
    fn draw_image(&mut self, image: &dyn CanvasImageSource, dx: f64, dy: f64) -> Result<()>;
    fn draw_image_scaled(
        &mut self,
        image: &dyn CanvasImageSource,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
    ) -> Result<()>;
    fn draw_image_subrect(
        &mut self,
        image: &dyn CanvasImageSource,
        sx: f64,
        sy: f64,
        sw: f64,
        sh: f64,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
    ) -> Result<()>;
}

pub trait CanvasPathDrawingStyles: CanvasLineStyles + CanvasFillStrokeStyles {}

pub trait CanvasImageSource {}

pub trait CanvasRenderingContext2D:
    CanvasState
    + CanvasTransforms
    + CanvasCompositing
    + CanvasRectangles
    + CanvasPaths
    + CanvasPathDrawingStyles
    + CanvasText
    + CanvasImageData
    + CanvasDrawImage
{
}

impl<T> CanvasPathDrawingStyles for T where T: CanvasLineStyles + CanvasFillStrokeStyles {}
