use vulkano::image::SampleCount;

enum Format {
    ColorUNormBgra8,
    ColorSFloatRgba16,
    ColorUNormRgba8,
    DepthSFloat32,
}

struct Attachment {
    dimensions: [u32; 2],
    format: Format,
}
impl Attachment {
    pub fn new(format: Format, dimensions: [u32; 2]) -> Self {
        Self { dimensions, format }
    }
}

struct RenderPass {
    samples: SampleCount,
}
impl RenderPass {
    pub fn new(samples: SampleCount) -> Self {
        Self { samples }
    }

    pub fn subpass(&self, name: &str) -> Subpass {
        Subpass {}
    }
}

struct VertexShader {}
impl VertexShader {
    pub fn load(path: &str) -> Self {
        Self {}
    }
}

struct FragmentShader {}
impl FragmentShader {
    pub fn load(path: &str) -> Self {
        Self {}
    }
}

struct Subpass {}
impl Subpass {
    fn with_vertex_shader(self, path: &str) -> Self {
        self
    }

    fn with_fragment_shader(self, path: &str) -> Self {
        self
    }

    fn render_color_to(self, attachment: &Attachment) -> Self {
        self
    }

    fn render_depth_to(self, attachment: &Attachment) -> Self {
        self
    }

    fn constant<Pc>(self, name: &str, value: Pc) -> Self {
        self
    }

    fn buffer<Buf>(self, name: &str, value: Buf) -> Self {
        self
    }

    fn vertices<Buf>(self, value: Buf) -> Self {
        self
    }

    fn indices<Buf>(self, value: Buf) -> Self {
        self
    }
}

struct Inputs {
    model_matrix: (),
    projection_matrix: (),

    opaque_surface_vertices: (),
    opaque_surface_indices: (),
    opaque_surface_materials: (),

    edge_vertices: (),
    edge_indices: (),

    point_lights: (),
    ambient_lights: (),
    directional_lights: (),
}

fn run(inputs: Inputs) {
    const FINAL_IMAGE_FORMAT: Format = Format::ColorUNormBgra8;
    const TRANSLUCENT_ACCUM_FORMAT: Format = Format::ColorSFloatRgba16;
    const TRANSLUCENT_TRANSMISSION_FORMAT: Format = Format::ColorUNormRgba8;
    const DEPTH_FORMAT: Format = Format::DepthSFloat32;

    let samples = SampleCount::Sample2;
    let dimensions = [1600, 900];

    let render_pass = RenderPass::new(samples);

    let opaque_image = Attachment::new(FINAL_IMAGE_FORMAT, dimensions);
    let depth_stencil = Attachment::new(DEPTH_FORMAT, dimensions);

    render_pass
        .subpass("opaque_surface")
        .constant("model_matrix", inputs.model_matrix)
        .constant("projection_matrix", inputs.projection_matrix)
        .buffer("point_light_buffer", inputs.point_lights)
        .buffer("ambient_light_buffer", inputs.ambient_lights)
        .buffer("directional_light_buffer", inputs.directional_lights)
        .buffer("material_buffer", inputs.opaque_surface_materials)
        .vertices(inputs.opaque_surface_vertices)
        .indices(inputs.opaque_surface_indices)
        .with_vertex_shader("surface.vert")
        .with_fragment_shader("opaque_surface.frag")
        .render_color_to(&opaque_image)
        .render_depth_to(&depth_stencil);

    render_pass
        .subpass("edge")
        .constant("model_matrix", inputs.model_matrix)
        .constant("projection_matrix", inputs.projection_matrix)
        .vertices(inputs.edge_vertices)
        .indices(inputs.edge_indices)
        .with_vertex_shader("edge.vert")
        .with_fragment_shader("edge.frag")
        .render_color_to(&opaque_image)
        .render_depth_to(&depth_stencil);

    todo!()
}
