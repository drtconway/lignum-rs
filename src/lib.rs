//! Traits and supporting types that mirror the HTML Canvas 2D context surface.
//! These are interface definitions only; you can implement them for any backend
//! (software rasterizer, OpenGL, WebGPU, etc.).

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
    Linear { x0: f64, y0: f64, x1: f64, y1: f64 },
    Radial { x0: f64, y0: f64, r0: f64, x1: f64, y1: f64, r1: f64 },
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
    fn save(&mut self);
    fn restore(&mut self);
    fn reset(&mut self);

    fn set_global_alpha(&mut self, value: f64);
    fn global_alpha(&self) -> f64;

    fn set_global_composite_operation(&mut self, op: CompositeOperation);
    fn global_composite_operation(&self) -> CompositeOperation;

    fn set_image_smoothing_enabled(&mut self, enabled: bool);
    fn image_smoothing_enabled(&self) -> bool;

    fn set_image_smoothing_quality(&mut self, quality: ImageSmoothingQuality);
    fn image_smoothing_quality(&self) -> ImageSmoothingQuality;
}

pub trait CanvasTransforms {
    fn scale(&mut self, x: f64, y: f64);
    fn rotate(&mut self, radians: f64);
    fn translate(&mut self, x: f64, y: f64);
    fn transform(&mut self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64);
    fn set_transform(&mut self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64);
    fn reset_transform(&mut self);
}

pub trait CanvasCompositing {
    fn set_shadow_offset_x(&mut self, value: f64);
    fn shadow_offset_x(&self) -> f64;

    fn set_shadow_offset_y(&mut self, value: f64);
    fn shadow_offset_y(&self) -> f64;

    fn set_shadow_blur(&mut self, value: f64);
    fn shadow_blur(&self) -> f64;

    fn set_shadow_color(&mut self, value: String);
    fn shadow_color(&self) -> String;
}

pub trait CanvasLineStyles {
    fn set_line_width(&mut self, value: f64);
    fn line_width(&self) -> f64;

    fn set_line_cap(&mut self, value: LineCap);
    fn line_cap(&self) -> LineCap;

    fn set_line_join(&mut self, value: LineJoin);
    fn line_join(&self) -> LineJoin;

    fn set_miter_limit(&mut self, value: f64);
    fn miter_limit(&self) -> f64;

    fn set_line_dash(&mut self, segments: Vec<f64>);
    fn line_dash(&self) -> Vec<f64>;

    fn set_line_dash_offset(&mut self, value: f64);
    fn line_dash_offset(&self) -> f64;
}

pub trait CanvasFillStrokeStyles {
    fn set_fill_style(&mut self, style: Paint);
    fn fill_style(&self) -> Paint;

    fn set_stroke_style(&mut self, style: Paint);
    fn stroke_style(&self) -> Paint;

    fn create_linear_gradient(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) -> CanvasGradient;
    fn create_radial_gradient(
        &mut self,
        x0: f64,
        y0: f64,
        r0: f64,
        x1: f64,
        y1: f64,
        r1: f64,
    ) -> CanvasGradient;

    fn create_pattern(
        &mut self,
        image: &dyn CanvasImageSource,
        repetition: PatternRepetition,
    ) -> CanvasPattern;
}

pub trait CanvasRectangles {
    fn clear_rect(&mut self, x: f64, y: f64, w: f64, h: f64);
    fn fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64);
    fn stroke_rect(&mut self, x: f64, y: f64, w: f64, h: f64);
}

pub trait CanvasPaths {
    fn begin_path(&mut self);
    fn close_path(&mut self);
    fn move_to(&mut self, x: f64, y: f64);
    fn line_to(&mut self, x: f64, y: f64);
    fn bezier_curve_to(&mut self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64);
    fn quadratic_curve_to(&mut self, cpx: f64, cpy: f64, x: f64, y: f64);
    fn arc(&mut self, x: f64, y: f64, radius: f64, start_angle: f64, end_angle: f64, ccw: bool);
    fn arc_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, radius: f64);
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
    );
    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64);
    fn round_rect(&mut self, x: f64, y: f64, w: f64, h: f64, radii: &[f64]);

    fn fill(&mut self, fill_rule: FillRule);
    fn stroke(&mut self);
    fn clip(&mut self, fill_rule: FillRule);
    fn is_point_in_path(&self, x: f64, y: f64, opts: HitOptions) -> bool;
    fn is_point_in_stroke(&self, x: f64, y: f64) -> bool;
}

pub trait CanvasText {
    fn set_font(&mut self, value: String);
    fn font(&self) -> String;

    fn set_text_align(&mut self, value: TextAlign);
    fn text_align(&self) -> TextAlign;

    fn set_text_baseline(&mut self, value: TextBaseline);
    fn text_baseline(&self) -> TextBaseline;

    fn set_direction(&mut self, value: Direction);
    fn direction(&self) -> Direction;

    fn fill_text(&mut self, text: &str, x: f64, y: f64, max_width: Option<f64>);
    fn stroke_text(&mut self, text: &str, x: f64, y: f64, max_width: Option<f64>);
    fn measure_text(&self, text: &str) -> TextMetrics;
}

pub trait CanvasImageData {
    fn create_image_data(&mut self, width: u32, height: u32) -> ImageData;
    fn get_image_data(&self, sx: u32, sy: u32, sw: u32, sh: u32) -> ImageData;
    fn put_image_data(&mut self, data: &ImageData, dx: f64, dy: f64);
    fn put_image_data_dirty(
        &mut self,
        data: &ImageData,
        dx: f64,
        dy: f64,
        dirty_x: u32,
        dirty_y: u32,
        dirty_width: u32,
        dirty_height: u32,
    );
}

pub trait CanvasDrawImage {
    fn draw_image(&mut self, image: &dyn CanvasImageSource, dx: f64, dy: f64);
    fn draw_image_scaled(&mut self, image: &dyn CanvasImageSource, dx: f64, dy: f64, dw: f64, dh: f64);
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
    );
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
