use crate::api::*;
use crate::error::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum PathCommand {
    MoveTo { x: f64, y: f64 },
    LineTo { x: f64, y: f64 },
    BezierCurveTo {
        cp1x: f64,
        cp1y: f64,
        cp2x: f64,
        cp2y: f64,
        x: f64,
        y: f64,
    },
    QuadraticCurveTo { cpx: f64, cpy: f64, x: f64, y: f64 },
    Arc {
        x: f64,
        y: f64,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        ccw: bool,
    },
    ArcTo {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        radius: f64,
    },
    Ellipse {
        x: f64,
        y: f64,
        radius_x: f64,
        radius_y: f64,
        rotation: f64,
        start_angle: f64,
        end_angle: f64,
        ccw: bool,
    },
    Rect { x: f64, y: f64, w: f64, h: f64 },
    RoundRect { x: f64, y: f64, w: f64, h: f64, radii: [f64; 4] },
    ClosePath,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordedPath {
    pub commands: Vec<PathCommand>,
}

impl RecordedPath {
    pub fn new(commands: Vec<PathCommand>) -> Self {
        Self { commands }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClipState {
    pub path: RecordedPath,
    pub rule: FillRule,
    pub transform: [f64; 6],
}

#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub global_alpha: f64,
    pub composite: CompositeOperation,
    pub image_smoothing_enabled: bool,
    pub image_smoothing_quality: ImageSmoothingQuality,
    pub shadow_offset_x: f64,
    pub shadow_offset_y: f64,
    pub shadow_blur: f64,
    pub shadow_color: String,
    pub line_width: f64,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub miter_limit: f64,
    pub line_dash: Vec<f64>,
    pub line_dash_offset: f64,
    pub fill_style: Paint,
    pub stroke_style: Paint,
    pub font: String,
    pub text_align: TextAlign,
    pub text_baseline: TextBaseline,
    pub direction: Direction,
    pub transform: [f64; 6],
    pub clip: Option<ClipState>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DrawOp {
    FillPath {
        path: RecordedPath,
        state: Snapshot,
        rule: FillRule,
    },
    StrokePath {
        path: RecordedPath,
        state: Snapshot,
    },
    Clip {
        path: RecordedPath,
        state: Snapshot,
        rule: FillRule,
    },
    FillRect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        state: Snapshot,
    },
    StrokeRect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        state: Snapshot,
    },
    FillText {
        text: String,
        x: f64,
        y: f64,
        max_width: Option<f64>,
        state: Snapshot,
    },
    StrokeText {
        text: String,
        x: f64,
        y: f64,
        max_width: Option<f64>,
        state: Snapshot,
    },
    DrawImage {
        source_width: u32,
        source_height: u32,
        dx: f64,
        dy: f64,
        state: Snapshot,
    },
    DrawImageScaled {
        source_width: u32,
        source_height: u32,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
        state: Snapshot,
    },
    DrawImageSubrect {
        source_width: u32,
        source_height: u32,
        sx: f64,
        sy: f64,
        sw: f64,
        sh: f64,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
        state: Snapshot,
    },
    PutImageData {
        data: ImageData,
        dx: f64,
        dy: f64,
        state: Snapshot,
    },
    PutImageDataDirty {
        data: ImageData,
        dx: f64,
        dy: f64,
        dirty_x: u32,
        dirty_y: u32,
        dirty_width: u32,
        dirty_height: u32,
        state: Snapshot,
    },
    ClearRect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        state: Snapshot,
    },
}

#[derive(Clone, Debug)]
struct RecorderState {
    global_alpha: f64,
    composite: CompositeOperation,
    image_smoothing_enabled: bool,
    image_smoothing_quality: ImageSmoothingQuality,
    shadow_offset_x: f64,
    shadow_offset_y: f64,
    shadow_blur: f64,
    shadow_color: String,
    line_width: f64,
    line_cap: LineCap,
    line_join: LineJoin,
    miter_limit: f64,
    line_dash: Vec<f64>,
    line_dash_offset: f64,
    fill_style: Paint,
    stroke_style: Paint,
    font: String,
    text_align: TextAlign,
    text_baseline: TextBaseline,
    direction: Direction,
    transform: [f64; 6],
    clip: Option<ClipState>,
}

impl Default for RecorderState {
    fn default() -> Self {
        Self {
            global_alpha: 1.0,
            composite: CompositeOperation::SourceOver,
            image_smoothing_enabled: true,
            image_smoothing_quality: ImageSmoothingQuality::Low,
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_blur: 0.0,
            shadow_color: "rgba(0,0,0,0)".to_string(),
            line_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            miter_limit: 10.0,
            line_dash: Vec::new(),
            line_dash_offset: 0.0,
            fill_style: Paint::Color("#000".to_string()),
            stroke_style: Paint::Color("#000".to_string()),
            font: "10px sans-serif".to_string(),
            text_align: TextAlign::Start,
            text_baseline: TextBaseline::Alphabetic,
            direction: Direction::Inherit,
            transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            clip: None,
        }
    }
}

pub struct RecordingCanvas {
    ops: Vec<DrawOp>,
    state: RecorderState,
    stack: Vec<RecorderState>,
    current_path: Vec<PathCommand>,
    current_point: Option<(f64, f64)>,
    subpath_start: Option<(f64, f64)>,
}

impl RecordingCanvas {
    pub fn new() -> Self {
        Self {
            ops: Vec::new(),
            state: RecorderState::default(),
            stack: Vec::new(),
            current_path: Vec::new(),
            current_point: None,
            subpath_start: None,
        }
    }

    pub fn ops(&self) -> &[DrawOp] {
        &self.ops
    }

    pub fn into_ops(self) -> Vec<DrawOp> {
        self.ops
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            global_alpha: self.state.global_alpha,
            composite: self.state.composite.clone(),
            image_smoothing_enabled: self.state.image_smoothing_enabled,
            image_smoothing_quality: self.state.image_smoothing_quality.clone(),
            shadow_offset_x: self.state.shadow_offset_x,
            shadow_offset_y: self.state.shadow_offset_y,
            shadow_blur: self.state.shadow_blur,
            shadow_color: self.state.shadow_color.clone(),
            line_width: self.state.line_width,
            line_cap: self.state.line_cap.clone(),
            line_join: self.state.line_join.clone(),
            miter_limit: self.state.miter_limit,
            line_dash: self.state.line_dash.clone(),
            line_dash_offset: self.state.line_dash_offset,
            fill_style: self.state.fill_style.clone(),
            stroke_style: self.state.stroke_style.clone(),
            font: self.state.font.clone(),
            text_align: self.state.text_align.clone(),
            text_baseline: self.state.text_baseline.clone(),
            direction: self.state.direction.clone(),
            transform: self.state.transform,
            clip: self.state.clip.clone(),
        }
    }

    fn ensure_subpath(&mut self) -> Result<()> {
        if self.current_point.is_none() {
            self.move_to(0.0, 0.0)?;
        }
        Ok(())
    }

    fn set_current_point(&mut self, x: f64, y: f64) {
        self.current_point = Some((x, y));
    }

    fn multiply_transform(&mut self, m: [f64; 6]) {
        let [a, b, c, d, e, f] = self.state.transform;
        let [na, nb, nc, nd, ne, nf] = m;
        self.state.transform = [
            a * na + c * nb,
            b * na + d * nb,
            a * nc + c * nd,
            b * nc + d * nd,
            a * ne + c * nf + e,
            b * ne + d * nf + f,
        ];
    }

    fn push_path(&mut self, cmd: PathCommand) {
        self.current_path.push(cmd);
    }

    fn consume_path(&mut self) -> RecordedPath {
        let path = RecordedPath::new(self.current_path.clone());
        self.current_path.clear();
        self.current_point = None;
        self.subpath_start = None;
        path
    }

    fn record_op(&mut self, op: DrawOp) {
        self.ops.push(op);
    }
}

impl Default for RecordingCanvas {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasState for RecordingCanvas {
    fn save(&mut self) -> Result<()> {
        self.stack.push(self.state.clone());
        Ok(())
    }

    fn restore(&mut self) -> Result<()> {
        if let Some(state) = self.stack.pop() {
            self.state = state;
        }
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        self.state = RecorderState::default();
        self.current_path.clear();
        self.current_point = None;
        self.subpath_start = None;
        Ok(())
    }

    fn set_global_alpha(&mut self, value: f64) -> Result<()> {
        self.state.global_alpha = value;
        Ok(())
    }

    fn global_alpha(&self) -> Result<f64> {
        Ok(self.state.global_alpha)
    }

    fn set_global_composite_operation(&mut self, op: CompositeOperation) -> Result<()> {
        self.state.composite = op;
        Ok(())
    }

    fn global_composite_operation(&self) -> Result<CompositeOperation> {
        Ok(self.state.composite.clone())
    }

    fn set_image_smoothing_enabled(&mut self, enabled: bool) -> Result<()> {
        self.state.image_smoothing_enabled = enabled;
        Ok(())
    }

    fn image_smoothing_enabled(&self) -> Result<bool> {
        Ok(self.state.image_smoothing_enabled)
    }

    fn set_image_smoothing_quality(&mut self, quality: ImageSmoothingQuality) -> Result<()> {
        self.state.image_smoothing_quality = quality;
        Ok(())
    }

    fn image_smoothing_quality(&self) -> Result<ImageSmoothingQuality> {
        Ok(self.state.image_smoothing_quality.clone())
    }
}

impl CanvasTransforms for RecordingCanvas {
    fn scale(&mut self, x: f64, y: f64) -> Result<()> {
        self.multiply_transform([x, 0.0, 0.0, y, 0.0, 0.0]);
        Ok(())
    }

    fn rotate(&mut self, radians: f64) -> Result<()> {
        let cos = radians.cos();
        let sin = radians.sin();
        self.multiply_transform([cos, sin, -sin, cos, 0.0, 0.0]);
        Ok(())
    }

    fn translate(&mut self, x: f64, y: f64) -> Result<()> {
        self.multiply_transform([1.0, 0.0, 0.0, 1.0, x, y]);
        Ok(())
    }

    fn transform(&mut self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Result<()> {
        self.multiply_transform([a, b, c, d, e, f]);
        Ok(())
    }

    fn set_transform(&mut self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Result<()> {
        self.state.transform = [a, b, c, d, e, f];
        Ok(())
    }

    fn reset_transform(&mut self) -> Result<()> {
        self.state.transform = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        Ok(())
    }
}

impl CanvasCompositing for RecordingCanvas {
    fn set_shadow_offset_x(&mut self, value: f64) -> Result<()> {
        self.state.shadow_offset_x = value;
        Ok(())
    }

    fn shadow_offset_x(&self) -> Result<f64> {
        Ok(self.state.shadow_offset_x)
    }

    fn set_shadow_offset_y(&mut self, value: f64) -> Result<()> {
        self.state.shadow_offset_y = value;
        Ok(())
    }

    fn shadow_offset_y(&self) -> Result<f64> {
        Ok(self.state.shadow_offset_y)
    }

    fn set_shadow_blur(&mut self, value: f64) -> Result<()> {
        self.state.shadow_blur = value;
        Ok(())
    }

    fn shadow_blur(&self) -> Result<f64> {
        Ok(self.state.shadow_blur)
    }

    fn set_shadow_color(&mut self, value: String) -> Result<()> {
        self.state.shadow_color = value;
        Ok(())
    }

    fn shadow_color(&self) -> Result<String> {
        Ok(self.state.shadow_color.clone())
    }
}

impl CanvasLineStyles for RecordingCanvas {
    fn set_line_width(&mut self, value: f64) -> Result<()> {
        self.state.line_width = value;
        Ok(())
    }

    fn line_width(&self) -> Result<f64> {
        Ok(self.state.line_width)
    }

    fn set_line_cap(&mut self, value: LineCap) -> Result<()> {
        self.state.line_cap = value;
        Ok(())
    }

    fn line_cap(&self) -> Result<LineCap> {
        Ok(self.state.line_cap.clone())
    }

    fn set_line_join(&mut self, value: LineJoin) -> Result<()> {
        self.state.line_join = value;
        Ok(())
    }

    fn line_join(&self) -> Result<LineJoin> {
        Ok(self.state.line_join.clone())
    }

    fn set_miter_limit(&mut self, value: f64) -> Result<()> {
        self.state.miter_limit = value;
        Ok(())
    }

    fn miter_limit(&self) -> Result<f64> {
        Ok(self.state.miter_limit)
    }

    fn set_line_dash(&mut self, segments: Vec<f64>) -> Result<()> {
        self.state.line_dash = segments;
        Ok(())
    }

    fn line_dash(&self) -> Result<Vec<f64>> {
        Ok(self.state.line_dash.clone())
    }

    fn set_line_dash_offset(&mut self, value: f64) -> Result<()> {
        self.state.line_dash_offset = value;
        Ok(())
    }

    fn line_dash_offset(&self) -> Result<f64> {
        Ok(self.state.line_dash_offset)
    }
}

impl CanvasFillStrokeStyles for RecordingCanvas {
    fn set_fill_style(&mut self, style: Paint) -> Result<()> {
        self.state.fill_style = style;
        Ok(())
    }

    fn fill_style(&self) -> Result<Paint> {
        Ok(self.state.fill_style.clone())
    }

    fn set_stroke_style(&mut self, style: Paint) -> Result<()> {
        self.state.stroke_style = style;
        Ok(())
    }

    fn stroke_style(&self) -> Result<Paint> {
        Ok(self.state.stroke_style.clone())
    }

    fn create_linear_gradient(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) -> Result<CanvasGradient> {
        Ok(CanvasGradient {
            kind: GradientKind::Linear { x0, y0, x1, y1 },
            stops: Vec::new(),
        })
    }

    fn create_radial_gradient(
        &mut self,
        x0: f64,
        y0: f64,
        r0: f64,
        x1: f64,
        y1: f64,
        r1: f64,
    ) -> Result<CanvasGradient> {
        Ok(CanvasGradient {
            kind: GradientKind::Radial {
                x0,
                y0,
                r0,
                x1,
                y1,
                r1,
            },
            stops: Vec::new(),
        })
    }

    fn create_pattern(
        &mut self,
        _image: &dyn CanvasImageSource,
        repetition: PatternRepetition,
    ) -> Result<CanvasPattern> {
        Ok(CanvasPattern {
            repetition,
            transform: None,
        })
    }
}

impl CanvasRectangles for RecordingCanvas {
    fn clear_rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> Result<()> {
        let op = DrawOp::ClearRect {
            x,
            y,
            w,
            h,
            state: self.snapshot(),
        };
        self.record_op(op);
        Ok(())
    }

    fn fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> Result<()> {
        let op = DrawOp::FillRect {
            x,
            y,
            w,
            h,
            state: self.snapshot(),
        };
        self.record_op(op);
        Ok(())
    }

    fn stroke_rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> Result<()> {
        let op = DrawOp::StrokeRect {
            x,
            y,
            w,
            h,
            state: self.snapshot(),
        };
        self.record_op(op);
        Ok(())
    }
}

impl CanvasPaths for RecordingCanvas {
    fn begin_path(&mut self) -> Result<()> {
        self.current_path.clear();
        self.current_point = None;
        self.subpath_start = None;
        Ok(())
    }

    fn close_path(&mut self) -> Result<()> {
        self.push_path(PathCommand::ClosePath);
        if let Some((x, y)) = self.subpath_start {
            self.set_current_point(x, y);
        }
        Ok(())
    }

    fn move_to(&mut self, x: f64, y: f64) -> Result<()> {
        self.push_path(PathCommand::MoveTo { x, y });
        self.subpath_start = Some((x, y));
        self.set_current_point(x, y);
        Ok(())
    }

    fn line_to(&mut self, x: f64, y: f64) -> Result<()> {
        if self.current_point.is_none() {
            self.move_to(0.0, 0.0)?;
        }
        self.push_path(PathCommand::LineTo { x, y });
        self.set_current_point(x, y);
        Ok(())
    }

    fn bezier_curve_to(
        &mut self,
        cp1x: f64,
        cp1y: f64,
        cp2x: f64,
        cp2y: f64,
        x: f64,
        y: f64,
    ) -> Result<()> {
        self.ensure_subpath()?;
        self.push_path(PathCommand::BezierCurveTo {
            cp1x,
            cp1y,
            cp2x,
            cp2y,
            x,
            y,
        });
        self.set_current_point(x, y);
        Ok(())
    }

    fn quadratic_curve_to(&mut self, cpx: f64, cpy: f64, x: f64, y: f64) -> Result<()> {
        self.ensure_subpath()?;
        self.push_path(PathCommand::QuadraticCurveTo { cpx, cpy, x, y });
        self.set_current_point(x, y);
        Ok(())
    }

    fn arc(
        &mut self,
        x: f64,
        y: f64,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        ccw: bool,
    ) -> Result<()> {
        // Record the arc; approximate the new current point at the end of the arc.
        self.ensure_subpath()?;
        self.push_path(PathCommand::Arc {
            x,
            y,
            radius,
            start_angle,
            end_angle,
            ccw,
        });
        let end_x = x + radius * end_angle.cos();
        let end_y = y + radius * end_angle.sin();
        self.set_current_point(end_x, end_y);
        Ok(())
    }

    fn arc_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, radius: f64) -> Result<()> {
        if self.current_point.is_none() {
            self.move_to(x1, y1)?;
        }
        self.push_path(PathCommand::ArcTo {
            x1,
            y1,
            x2,
            y2,
            radius,
        });
        self.set_current_point(x2, y2);
        Ok(())
    }

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
    ) -> Result<()> {
        self.ensure_subpath()?;
        self.push_path(PathCommand::Ellipse {
            x,
            y,
            radius_x,
            radius_y,
            rotation,
            start_angle,
            end_angle,
            ccw,
        });
        let cos_r = rotation.cos();
        let sin_r = rotation.sin();
        let ex = radius_x * end_angle.cos();
        let ey = radius_y * end_angle.sin();
        let end_x = x + ex * cos_r - ey * sin_r;
        let end_y = y + ex * sin_r + ey * cos_r;
        self.set_current_point(end_x, end_y);
        Ok(())
    }

    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> Result<()> {
        self.push_path(PathCommand::Rect { x, y, w, h });
        self.subpath_start = Some((x, y));
        self.set_current_point(x, y);
        Ok(())
    }

    fn round_rect(&mut self, x: f64, y: f64, w: f64, h: f64, radii: &[f64]) -> Result<()> {
        let mut corner = [0.0; 4];
        match radii.len() {
            0 => {}
            1 => corner.fill(radii[0]),
            2 => {
                corner[0] = radii[0];
                corner[1] = radii[1];
                corner[2] = radii[0];
                corner[3] = radii[1];
            }
            3 => {
                corner[0] = radii[0];
                corner[1] = radii[1];
                corner[2] = radii[2];
                corner[3] = radii[1];
            }
            _ => {
                corner[0] = radii[0];
                corner[1] = radii[1];
                corner[2] = radii[2];
                corner[3] = radii[3];
            }
        }
        self.push_path(PathCommand::RoundRect {
            x,
            y,
            w,
            h,
            radii: corner,
        });
        self.subpath_start = Some((x, y));
        self.set_current_point(x, y);
        Ok(())
    }

    fn fill(&mut self, fill_rule: FillRule) -> Result<()> {
        if self.current_path.is_empty() {
            return Ok(());
        }
        let path = self.consume_path();
        let op = DrawOp::FillPath {
            path,
            state: self.snapshot(),
            rule: fill_rule,
        };
        self.record_op(op);
        Ok(())
    }

    fn stroke(&mut self) -> Result<()> {
        if self.current_path.is_empty() {
            return Ok(());
        }
        let path = self.consume_path();
        let op = DrawOp::StrokePath {
            path,
            state: self.snapshot(),
        };
        self.record_op(op);
        Ok(())
    }

    fn clip(&mut self, fill_rule: FillRule) -> Result<()> {
        if self.current_path.is_empty() {
            return Ok(());
        }
        let path = self.consume_path();
        let clip_state = ClipState {
            path: path.clone(),
            rule: fill_rule.clone(),
            transform: self.state.transform,
        };
        self.state.clip = Some(clip_state.clone());
        let op = DrawOp::Clip {
            path,
            state: self.snapshot(),
            rule: fill_rule,
        };
        self.record_op(op);
        Ok(())
    }

    fn is_point_in_path(&self, _x: f64, _y: f64, _opts: HitOptions) -> Result<bool> {
        Ok(false)
    }

    fn is_point_in_stroke(&self, _x: f64, _y: f64) -> Result<bool> {
        Ok(false)
    }
}

impl CanvasText for RecordingCanvas {
    fn set_font(&mut self, value: String) -> Result<()> {
        self.state.font = value;
        Ok(())
    }

    fn font(&self) -> Result<String> {
        Ok(self.state.font.clone())
    }

    fn set_text_align(&mut self, value: TextAlign) -> Result<()> {
        self.state.text_align = value;
        Ok(())
    }

    fn text_align(&self) -> Result<TextAlign> {
        Ok(self.state.text_align.clone())
    }

    fn set_text_baseline(&mut self, value: TextBaseline) -> Result<()> {
        self.state.text_baseline = value;
        Ok(())
    }

    fn text_baseline(&self) -> Result<TextBaseline> {
        Ok(self.state.text_baseline.clone())
    }

    fn set_direction(&mut self, value: Direction) -> Result<()> {
        self.state.direction = value;
        Ok(())
    }

    fn direction(&self) -> Result<Direction> {
        Ok(self.state.direction.clone())
    }

    fn fill_text(&mut self, text: &str, x: f64, y: f64, max_width: Option<f64>) -> Result<()> {
        let op = DrawOp::FillText {
            text: text.to_string(),
            x,
            y,
            max_width,
            state: self.snapshot(),
        };
        self.record_op(op);
        Ok(())
    }

    fn stroke_text(&mut self, text: &str, x: f64, y: f64, max_width: Option<f64>) -> Result<()> {
        let op = DrawOp::StrokeText {
            text: text.to_string(),
            x,
            y,
            max_width,
            state: self.snapshot(),
        };
        self.record_op(op);
        Ok(())
    }

    fn measure_text(&self, text: &str) -> Result<TextMetrics> {
        Ok(TextMetrics {
            width: text.len() as f64,
        })
    }
}

impl CanvasImageData for RecordingCanvas {
    fn create_image_data(&mut self, width: u32, height: u32) -> Result<ImageData> {
        Ok(ImageData {
            width,
            height,
            data: vec![0; (width * height * 4) as usize],
        })
    }

    fn get_image_data(&self, sx: u32, sy: u32, sw: u32, sh: u32) -> Result<ImageData> {
        let _ = (sx, sy);
        Ok(ImageData {
            width: sw,
            height: sh,
            data: vec![0; (sw * sh * 4) as usize],
        })
    }

    fn put_image_data(&mut self, data: &ImageData, dx: f64, dy: f64) -> Result<()> {
        let op = DrawOp::PutImageData {
            data: data.clone(),
            dx,
            dy,
            state: self.snapshot(),
        };
        self.record_op(op);
        Ok(())
    }

    fn put_image_data_dirty(
        &mut self,
        data: &ImageData,
        dx: f64,
        dy: f64,
        dirty_x: u32,
        dirty_y: u32,
        dirty_width: u32,
        dirty_height: u32,
    ) -> Result<()> {
        let op = DrawOp::PutImageDataDirty {
            data: data.clone(),
            dx,
            dy,
            dirty_x,
            dirty_y,
            dirty_width,
            dirty_height,
            state: self.snapshot(),
        };
        self.record_op(op);
        Ok(())
    }
}

impl CanvasDrawImage for RecordingCanvas {
    fn draw_image(&mut self, image: &dyn CanvasImageSource, dx: f64, dy: f64) -> Result<()> {
        let op = DrawOp::DrawImage {
            source_width: image.width(),
            source_height: image.height(),
            dx,
            dy,
            state: self.snapshot(),
        };
        self.record_op(op);
        Ok(())
    }

    fn draw_image_scaled(
        &mut self,
        image: &dyn CanvasImageSource,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
    ) -> Result<()> {
        let op = DrawOp::DrawImageScaled {
            source_width: image.width(),
            source_height: image.height(),
            dx,
            dy,
            dw,
            dh,
            state: self.snapshot(),
        };
        self.record_op(op);
        Ok(())
    }

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
    ) -> Result<()> {
        let op = DrawOp::DrawImageSubrect {
            source_width: image.width(),
            source_height: image.height(),
            sx,
            sy,
            sw,
            sh,
            dx,
            dy,
            dw,
            dh,
            state: self.snapshot(),
        };
        self.record_op(op);
        Ok(())
    }
}

impl CanvasRenderingContext2D for RecordingCanvas {}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_almost_eq(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{} != {}", a, b);
    }

    #[test]
    fn records_fill_rect() {
        let mut c = RecordingCanvas::new();
        c.set_fill_style(Paint::Color("#f00".into())).unwrap();
        c.fill_rect(1.0, 2.0, 3.0, 4.0).unwrap();
        let ops = c.ops();
        assert_eq!(ops.len(), 1);
        match &ops[0] {
            DrawOp::FillRect { x, y, w, h, state } => {
                assert_almost_eq(*x, 1.0);
                assert_almost_eq(*y, 2.0);
                assert_almost_eq(*w, 3.0);
                assert_almost_eq(*h, 4.0);
                assert_eq!(state.fill_style, Paint::Color("#f00".into()));
                assert_eq!(state.transform, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
            }
            _ => panic!("unexpected op"),
        }
    }

    #[test]
    fn records_clip_and_fill_path() {
        let mut c = RecordingCanvas::new();
        c.begin_path().unwrap();
        c.move_to(0.0, 0.0).unwrap();
        c.line_to(10.0, 0.0).unwrap();
        c.line_to(10.0, 10.0).unwrap();
        c.clip(FillRule::EvenOdd).unwrap();

        c.begin_path().unwrap();
        c.rect(1.0, 1.0, 2.0, 2.0).unwrap();
        c.fill(FillRule::NonZero).unwrap();

        let ops = c.ops();
        assert_eq!(ops.len(), 2);

        match &ops[0] {
            DrawOp::Clip { path, rule, state } => {
                assert_eq!(*rule, FillRule::EvenOdd);
                assert_eq!(path.commands.len(), 3);
                assert_eq!(state.clip.as_ref().unwrap().rule, FillRule::EvenOdd);
            }
            _ => panic!("unexpected op"),
        }

        match &ops[1] {
            DrawOp::FillPath { path, state, rule } => {
                assert_eq!(path.commands.len(), 1);
                assert_eq!(*rule, FillRule::NonZero);
                assert!(state.clip.is_some());
            }
            _ => panic!("unexpected op"),
        }
    }

    #[test]
    fn records_transforms() {
        let mut c = RecordingCanvas::new();
        c.translate(5.0, 6.0).unwrap();
        c.scale(2.0, 3.0).unwrap();
        c.begin_path().unwrap();
        c.move_to(0.0, 0.0).unwrap();
        c.line_to(1.0, 1.0).unwrap();
        c.stroke().unwrap();

        let ops = c.ops();
        match &ops[0] {
            DrawOp::StrokePath { state, .. } => {
                assert_eq!(state.transform, [2.0, 0.0, 0.0, 3.0, 5.0, 6.0]);
            }
            _ => panic!("unexpected op"),
        }
    }
}
