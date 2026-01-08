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
    /// Saves all current drawing state attributes onto a stack (transform, styles, clipping, etc.). Mirrors CanvasRenderingContext2D.save().
    fn save(&mut self) -> Result<()>;
    /// Pops the last saved state from the stack and restores it. Mirrors CanvasRenderingContext2D.restore().
    fn restore(&mut self) -> Result<()>;
    /// Resets the state to defaults (as if a new context) without touching the stack. Mirrors CanvasRenderingContext2D.reset().
    fn reset(&mut self) -> Result<()>;

    /// Sets the global alpha multiplier applied to all drawing ops. Mirrors globalAlpha.
    fn set_global_alpha(&mut self, value: f64) -> Result<()>;
    /// Returns the current global alpha multiplier. Mirrors globalAlpha.
    fn global_alpha(&self) -> Result<f64>;

    /// Sets the Porter-Duff compositing operation used when drawing. Mirrors globalCompositeOperation.
    fn set_global_composite_operation(&mut self, op: CompositeOperation) -> Result<()>;
    /// Returns the current compositing operation. Mirrors globalCompositeOperation.
    fn global_composite_operation(&self) -> Result<CompositeOperation>;

    /// Enables or disables image smoothing for drawImage and scaling operations. Mirrors imageSmoothingEnabled.
    fn set_image_smoothing_enabled(&mut self, enabled: bool) -> Result<()>;
    /// Returns whether image smoothing is enabled. Mirrors imageSmoothingEnabled.
    fn image_smoothing_enabled(&self) -> Result<bool>;

    /// Sets the image smoothing quality hint (low/medium/high). Mirrors imageSmoothingQuality.
    fn set_image_smoothing_quality(&mut self, quality: ImageSmoothingQuality) -> Result<()>;
    /// Returns the current image smoothing quality hint. Mirrors imageSmoothingQuality.
    fn image_smoothing_quality(&self) -> Result<ImageSmoothingQuality>;
}

pub trait CanvasTransforms {
    /// Multiplies the current transform by a scaling matrix. Mirrors scale().
    fn scale(&mut self, x: f64, y: f64) -> Result<()>;
    /// Rotates the current transform by the given radians about the origin. Mirrors rotate().
    fn rotate(&mut self, radians: f64) -> Result<()>;
    /// Translates the current transform by (x, y). Mirrors translate().
    fn translate(&mut self, x: f64, y: f64) -> Result<()>;
    /// Multiplies the current transform by the provided matrix components. Mirrors transform().
    fn transform(&mut self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Result<()>;
    /// Replaces the current transform with the provided matrix. Mirrors setTransform().
    fn set_transform(&mut self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Result<()>;
    /// Resets the transform to the identity matrix. Mirrors resetTransform().
    fn reset_transform(&mut self) -> Result<()>;
}

pub trait CanvasCompositing {
    /// Sets the horizontal offset of the shadow blur. Mirrors shadowOffsetX.
    fn set_shadow_offset_x(&mut self, value: f64) -> Result<()>;
    /// Returns the horizontal shadow offset. Mirrors shadowOffsetX.
    fn shadow_offset_x(&self) -> Result<f64>;

    /// Sets the vertical offset of the shadow blur. Mirrors shadowOffsetY.
    fn set_shadow_offset_y(&mut self, value: f64) -> Result<()>;
    /// Returns the vertical shadow offset. Mirrors shadowOffsetY.
    fn shadow_offset_y(&self) -> Result<f64>;

    /// Sets the blur radius for shadows. Mirrors shadowBlur.
    fn set_shadow_blur(&mut self, value: f64) -> Result<()>;
    /// Returns the blur radius for shadows. Mirrors shadowBlur.
    fn shadow_blur(&self) -> Result<f64>;

    /// Sets the shadow color string. Mirrors shadowColor.
    fn set_shadow_color(&mut self, value: String) -> Result<()>;
    /// Returns the current shadow color string. Mirrors shadowColor.
    fn shadow_color(&self) -> Result<String>;
}

pub trait CanvasLineStyles {
    /// Sets stroke thickness in user units. Mirrors lineWidth.
    fn set_line_width(&mut self, value: f64) -> Result<()>;
    /// Returns the current stroke thickness. Mirrors lineWidth.
    fn line_width(&self) -> Result<f64>;

    /// Sets the shape of the line end caps. Mirrors lineCap.
    fn set_line_cap(&mut self, value: LineCap) -> Result<()>;
    /// Returns the current line cap. Mirrors lineCap.
    fn line_cap(&self) -> Result<LineCap>;

    /// Sets the shape used at line joins. Mirrors lineJoin.
    fn set_line_join(&mut self, value: LineJoin) -> Result<()>;
    /// Returns the current line join. Mirrors lineJoin.
    fn line_join(&self) -> Result<LineJoin>;

    /// Sets the miter limit used for miter joins. Mirrors miterLimit.
    fn set_miter_limit(&mut self, value: f64) -> Result<()>;
    /// Returns the current miter limit. Mirrors miterLimit.
    fn miter_limit(&self) -> Result<f64>;

    /// Sets the line dash pattern segments. Mirrors setLineDash().
    fn set_line_dash(&mut self, segments: Vec<f64>) -> Result<()>;
    /// Returns the current line dash segments. Mirrors getLineDash().
    fn line_dash(&self) -> Result<Vec<f64>>;

    /// Sets the offset into the dash pattern. Mirrors lineDashOffset.
    fn set_line_dash_offset(&mut self, value: f64) -> Result<()>;
    /// Returns the dash offset. Mirrors lineDashOffset.
    fn line_dash_offset(&self) -> Result<f64>;
}

pub trait CanvasFillStrokeStyles {
    /// Sets the paint used for fills (color/gradient/pattern). Mirrors fillStyle.
    fn set_fill_style(&mut self, style: Paint) -> Result<()>;
    /// Returns the current fill paint. Mirrors fillStyle.
    fn fill_style(&self) -> Result<Paint>;

    /// Sets the paint used for strokes. Mirrors strokeStyle.
    fn set_stroke_style(&mut self, style: Paint) -> Result<()>;
    /// Returns the current stroke paint. Mirrors strokeStyle.
    fn stroke_style(&self) -> Result<Paint>;

    /// Creates a linear gradient object. Mirrors createLinearGradient().
    fn create_linear_gradient(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) -> Result<CanvasGradient>;
    /// Creates a radial gradient object. Mirrors createRadialGradient().
    fn create_radial_gradient(
        &mut self,
        x0: f64,
        y0: f64,
        r0: f64,
        x1: f64,
        y1: f64,
        r1: f64,
    ) -> Result<CanvasGradient>;

    /// Creates a pattern from an image source with repetition behavior. Mirrors createPattern().
    fn create_pattern(
        &mut self,
        image: &dyn CanvasImageSource,
        repetition: PatternRepetition,
    ) -> Result<CanvasPattern>;
}

pub trait CanvasRectangles {
    /// Clears the specified rectangle to full transparency. Mirrors clearRect().
    fn clear_rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> Result<()>;
    /// Fills the specified rectangle using the current fill style. Mirrors fillRect().
    fn fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> Result<()>;
    /// Strokes the outline of the specified rectangle using the current stroke style. Mirrors strokeRect().
    fn stroke_rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> Result<()>;
}

pub trait CanvasPaths {
    /// Starts a new empty path list. Mirrors beginPath().
    fn begin_path(&mut self) -> Result<()>;
    /// Closes the current subpath with a straight line. Mirrors closePath().
    fn close_path(&mut self) -> Result<()>;
    /// Moves the current point without drawing. Mirrors moveTo().
    fn move_to(&mut self, x: f64, y: f64) -> Result<()>;
    /// Adds a straight line from the current point to (x, y). Mirrors lineTo().
    fn line_to(&mut self, x: f64, y: f64) -> Result<()>;
    /// Adds a cubic Bezier curve. Mirrors bezierCurveTo().
    fn bezier_curve_to(&mut self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) -> Result<()>;
    /// Adds a quadratic Bezier curve. Mirrors quadraticCurveTo().
    fn quadratic_curve_to(&mut self, cpx: f64, cpy: f64, x: f64, y: f64) -> Result<()>;
    /// Adds an arc centered at (x, y) with radius and angles. Mirrors arc().
    fn arc(&mut self, x: f64, y: f64, radius: f64, start_angle: f64, end_angle: f64, ccw: bool) -> Result<()>;
    /// Adds an arc that smoothly connects a line to another line. Mirrors arcTo().
    fn arc_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, radius: f64) -> Result<()>;
    /// Adds a rotated ellipse arc segment. Mirrors ellipse().
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
    /// Adds a rect subpath. Mirrors rect().
    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> Result<()>;
    /// Adds a rounded-rectangle subpath. Mirrors roundRect().
    fn round_rect(&mut self, x: f64, y: f64, w: f64, h: f64, radii: &[f64]) -> Result<()>;

    /// Fills the current path using the given fill rule. Mirrors fill().
    fn fill(&mut self, fill_rule: FillRule) -> Result<()>;
    /// Strokes the current path. Mirrors stroke().
    fn stroke(&mut self) -> Result<()>;
    /// Sets the current clipping region to the current path (optionally using a fill rule). Mirrors clip().
    fn clip(&mut self, fill_rule: FillRule) -> Result<()>;
    /// Reports whether the point lies within the filled region of the current path. Mirrors isPointInPath().
    fn is_point_in_path(&self, x: f64, y: f64, opts: HitOptions) -> Result<bool>;
    /// Reports whether the point lies within the stroked region of the current path. Mirrors isPointInStroke().
    fn is_point_in_stroke(&self, x: f64, y: f64) -> Result<bool>;
}

pub trait CanvasText {
    /// Sets the CSS font string used for text rendering. Mirrors font.
    fn set_font(&mut self, value: String) -> Result<()>;
    /// Returns the current font string. Mirrors font.
    fn font(&self) -> Result<String>;

    /// Sets horizontal text alignment relative to the anchor point. Mirrors textAlign.
    fn set_text_align(&mut self, value: TextAlign) -> Result<()>;
    /// Returns the current text alignment. Mirrors textAlign.
    fn text_align(&self) -> Result<TextAlign>;

    /// Sets the baseline alignment for text. Mirrors textBaseline.
    fn set_text_baseline(&mut self, value: TextBaseline) -> Result<()>;
    /// Returns the current text baseline. Mirrors textBaseline.
    fn text_baseline(&self) -> Result<TextBaseline>;

    /// Sets text direction (ltr/rtl/inherit). Mirrors direction.
    fn set_direction(&mut self, value: Direction) -> Result<()>;
    /// Returns the current text direction. Mirrors direction.
    fn direction(&self) -> Result<Direction>;

    /// Fills the given text at (x, y), optionally constraining to max width. Mirrors fillText().
    fn fill_text(&mut self, text: &str, x: f64, y: f64, max_width: Option<f64>) -> Result<()>;
    /// Strokes the given text at (x, y), optionally constraining to max width. Mirrors strokeText().
    fn stroke_text(&mut self, text: &str, x: f64, y: f64, max_width: Option<f64>) -> Result<()>;
    /// Measures the advance width of the given text using current font settings. Mirrors measureText().
    fn measure_text(&self, text: &str) -> Result<TextMetrics>;
}

pub trait CanvasImageData {
    /// Creates a blank ImageData object with the given dimensions. Mirrors createImageData(width, height).
    fn create_image_data(&mut self, width: u32, height: u32) -> Result<ImageData>;
    /// Returns ImageData for the given rectangle of the drawing surface. Mirrors getImageData().
    fn get_image_data(&self, sx: u32, sy: u32, sw: u32, sh: u32) -> Result<ImageData>;
    /// Paints the provided ImageData at (dx, dy). Mirrors putImageData().
    fn put_image_data(&mut self, data: &ImageData, dx: f64, dy: f64) -> Result<()>;
    /// Paints a dirty rect subset of ImageData at (dx, dy). Mirrors putImageData() with dirty rect.
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
    /// Draws the full source image with its intrinsic size at (dx, dy). Mirrors drawImage(image, dx, dy).
    fn draw_image(&mut self, image: &dyn CanvasImageSource, dx: f64, dy: f64) -> Result<()>;
    /// Draws and scales the source image to the given destination size. Mirrors drawImage(image, dx, dy, dw, dh).
    fn draw_image_scaled(
        &mut self,
        image: &dyn CanvasImageSource,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
    ) -> Result<()>;
    /// Draws a source sub-rectangle into a destination rectangle. Mirrors drawImage(image, sx, sy, sw, sh, dx, dy, dw, dh).
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

pub trait CanvasImageSource {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    /// Returns a view over RGBA pixels (premultiplied or straight alpha depending on backend expectations).
    /// Length must be width * height * 4.
    fn data_rgba(&self) -> Option<&[u8]>;
}

impl CanvasImageSource for ImageData {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn data_rgba(&self) -> Option<&[u8]> {
        Some(self.data.as_slice())
    }
}

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
