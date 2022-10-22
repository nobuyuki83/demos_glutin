use crate::gl;

struct ElementBufferObject {
    mode: gl::types::GLenum,
    elem_size: usize,
    ebo: gl::types::GLuint,
}

pub struct DrawerMeshPosColor {
    pub color_map: Vec<[f32;3]>,
    pub val_min: f32,
    pub val_max: f32,
    pub ndim: i32,
    program: gl::types::GLuint,
    vao: gl::types::GLuint,
    // uniform variables
    loc_mat_modelview : gl::types::GLint,
    loc_mat_projection: gl::types::GLint,
    loc_val_min: gl::types::GLint,
    loc_val_max: gl::types::GLint,
    // elemenb buffer object
    ebo: ElementBufferObject,
}

impl DrawerMeshPosColor {
    pub fn new() -> Self {
        DrawerMeshPosColor {
            color_map: Vec::<[f32;3]>::new(),
            val_min: 0.0,
            val_max: 1.0,
            program: 0,
            ndim: 0,
            vao: 0,
            loc_mat_modelview: -1,
            loc_mat_projection: -1,
            loc_val_min: -1,
            loc_val_max: -1,
            ebo: ElementBufferObject{
                mode: gl::TRIANGLES,
                elem_size: 0,
                ebo: 0,
            },
        }
    }
    pub fn compile_shader(&mut self, gl: &gl::Gl) {
        const VS_SRC: &[u8] = b"
#version 330

uniform mat4 matMV;
uniform mat4 matPrj;

layout (location = 0) in vec3 position;
layout (location = 1) in float value;
out float val;

void main() {
    gl_Position = matPrj * matMV * vec4(position, 1.0);
    // gl_Position = vec4(position, 1.0);
    val = value;
}
\0";

        let mut glsl_colormap = String::new();
        {
            glsl_colormap += &format!("const int ncolor = {};\n", self.color_map.len());
            glsl_colormap += "vec3[ncolor] colors = vec3[] (\n";
            for ic in 0..self.color_map.len() {
                let c = self.color_map[ic];
                glsl_colormap += &format!(" vec3({},{},{})", c[0], c[1], c[2]);
                if ic != self.color_map.len()-1 {  glsl_colormap += &",\n";  }
                else{ glsl_colormap += &");\n"; }
            }
        }

        let glsl_header: String = "
#version 330
".to_string();

        let glsl_code: String = "
uniform vec3 color;

uniform float val_min;
uniform float val_max;
in float val;

out vec4 FragColor;

void main() {
    float scaled_value = (val-val_min)/(val_max-val_min) * (ncolor-1);
    int idx_color = int(scaled_value);
    float r01 = scaled_value - float(idx_color);
    if( idx_color < 0 ){ idx_color = 0; r01 = 0.;}
    if( idx_color > ncolor-2 ){ idx_color = ncolor-2; r01 = 1.; }
    vec3 clr01 = (1.f-r01)*colors[idx_color] + r01*colors[idx_color+1];
    FragColor = vec4(clr01.x, clr01.y, clr01.z, 1.0f);
    // FragColor = vec4(color, 1.0);
}
\0".to_string();
        let fs_src = glsl_header + &glsl_colormap + &glsl_code;

        unsafe {
            let vs = gl.CreateShader(gl::VERTEX_SHADER);
            gl.ShaderSource(vs, 1, [VS_SRC.as_ptr() as *const _].as_ptr(), std::ptr::null());
            gl.CompileShader(vs);

            let fs = gl.CreateShader(gl::FRAGMENT_SHADER);
            gl.ShaderSource(fs, 1, [fs_src.as_ptr() as *const _].as_ptr(), std::ptr::null());
            gl.CompileShader(fs);

            self.program = gl.CreateProgram();
            gl.AttachShader(self.program, vs);
            gl.AttachShader(self.program, fs);
            gl.LinkProgram(self.program);
            assert!( gl.IsProgram(self.program) != 0 );
            {
                let mut success: gl::types::GLint = 0;
                gl.GetProgramiv(self.program, gl::LINK_STATUS, &mut success);
                if success == 0 {
                    let info_log: [i8; 512] = [0; 512];
                    let mut length: i32 = 512;
                    gl.GetProgramInfoLog(self.program, 512, &mut length, info_log.as_ptr() as *mut _);
                    println!("{}", length);
                    let info_log0 = String::from_utf8(info_log.iter().map(|&c| c as u8).collect());
                    println!("ERROR::SHADER::PROGRAM::LINKING_FAILED {:?}", info_log0);
                }
            }
            gl.DeleteShader(vs);
            gl.DeleteShader(fs);
        }

        unsafe { // make VAO
            if gl.BindVertexArray.is_loaded() {
                let mut vao0 = std::mem::zeroed();
                gl.GenVertexArrays(1, &mut vao0);
                self.vao = vao0;
                gl.BindVertexArray(self.vao);
            }
        }

        unsafe {  // locate uniform variables, should come after VAO is made
            {
                let cname = std::ffi::CString::new("matMV").expect("CString::new failed");
                self.loc_mat_modelview = gl.GetUniformLocation(self.program, cname.as_ptr());
            }
            {
                let cname = std::ffi::CString::new("matPrj").expect("CString::new failed");
                self.loc_mat_projection = gl.GetUniformLocation(self.program, cname.as_ptr());
            }
            {
                let cname = std::ffi::CString::new("val_min").expect("CString::new failed");
                self.loc_val_min = gl.GetUniformLocation(self.program, cname.as_ptr());
            }
            {
                let cname = std::ffi::CString::new("val_max").expect("CString::new failed");
                self.loc_val_max = gl.GetUniformLocation(self.program, cname.as_ptr());
            }
        }

    }

    pub fn add_element(
        &mut self,
        gl: &gl::Gl,
        mode: gl::types::GLenum,
        elem_vtx: &Vec<gl::types::GLuint>) {
        unsafe {
            gl.BindVertexArray(self.vao);
            let mut ebo0 = std::mem::zeroed();
            gl.GenBuffers(1, &mut ebo0);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo0);
            gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (elem_vtx.len() * std::mem::size_of::<usize>()) as gl::types::GLsizeiptr,
                elem_vtx.as_ptr() as *const _,
                gl::STATIC_DRAW);
            self.ebo.mode = mode;
            self.ebo.elem_size = elem_vtx.len();
            self.ebo.ebo = ebo0;
        }
    }

    pub fn update_vertex(
        &mut self,
        gl: &gl::Gl,
        vtx_xyz: &Vec<f32>,
        ndim: i32) {
        self.ndim = ndim;
        unsafe {
            gl.BindVertexArray(self.vao);

            let mut vbo = std::mem::zeroed();
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (vtx_xyz.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vtx_xyz.as_ptr() as *const _,
                gl::STATIC_DRAW);
            let pos_attrib = gl.GetAttribLocation(self.program, b"position\0".as_ptr() as *const _);
            gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            gl.VertexAttribPointer(
                pos_attrib as gl::types::GLuint,
                self.ndim,
                gl::FLOAT,
                0,
                self.ndim * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null());
        }
    }

    pub fn update_value(
        &mut self,
        gl: &gl::Gl,
        vtx_val: &Vec<f32>) {
        unsafe {
            gl.BindVertexArray(self.vao);
            let mut vbo = std::mem::zeroed();
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (vtx_val.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vtx_val.as_ptr() as *const _,
                gl::STATIC_DRAW);
            let val_attrib = gl.GetAttribLocation(self.program, b"value\0".as_ptr() as *const _);
            gl.EnableVertexAttribArray(val_attrib as gl::types::GLuint);
            gl.VertexAttribPointer(
                val_attrib as gl::types::GLuint,
                1,
                gl::FLOAT,
                0,
                1 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null());
        }
    }

    pub fn draw(
        &self,
        gl: &gl::Gl,
        mat_modelview: &[f32],
        mat_projection: &[f32]){
        let mp0 = mat_projection;
        let mp1: [f32;16] = [ // mp1 = [z flip] * mp0
            mp0[0], mp0[1], -mp0[2], mp0[3],
            mp0[4], mp0[5], -mp0[6], mp0[7],
            mp0[8], mp0[9], -mp0[10], mp0[11],
            mp0[12], mp0[13], -mp0[14], mp0[15] ];
        unsafe {
            gl.UseProgram(self.program);
            gl.BindVertexArray(self.vao);
            gl.UniformMatrix4fv(self.loc_mat_modelview, 1, gl::FALSE, mat_modelview.as_ptr());
            gl.UniformMatrix4fv(self.loc_mat_projection, 1, gl::FALSE, mp1.as_ptr());
            gl.Uniform1f(self.loc_val_min, self.val_min);
            gl.Uniform1f(self.loc_val_max, self.val_max);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo.ebo);
            gl.DrawElements(self.ebo.mode,
                            self.ebo.elem_size as i32,
                            gl::UNSIGNED_INT,
                            std::ptr::null());
        }
    }
}