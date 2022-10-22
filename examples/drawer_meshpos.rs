use crate::gl;

struct ElementBufferObject {
    mode: gl::types::GLenum,
    elem_size: usize,
    ebo: gl::types::GLuint,
    color: [f32;3]
}

pub struct DrawerMeshPos {
    program: gl::types::GLuint,
    pub ndim: i32,
    num_point: i32,
    vao: gl::types::GLuint,
    // uniform variables
    loc_color: gl::types::GLint,
    loc_mat_modelview : gl::types::GLint,
    loc_mat_projection: gl::types::GLint,
    // elemenb buffer object
    ebos: Vec<ElementBufferObject>,
}

impl DrawerMeshPos {
    pub fn new() -> Self {
        DrawerMeshPos {
            program: 0,
            ndim: 0,
            num_point: 0,
            vao: 0,
            loc_color: -1, // -1 is the failure flag
            loc_mat_modelview: -1,
            loc_mat_projection: -1,
            ebos: Vec::<ElementBufferObject>::new(),
        }
    }
    pub fn compile_shader(&mut self, gl: &gl::Gl) {
        const VS_SRC: &[u8] = b"
#version 330

uniform mat4 matMV;
uniform mat4 matPrj;

layout (location = 0) in vec3 position;

void main() {
    gl_Position = matPrj * matMV * vec4(position, 1.0);
    // gl_Position = vec4(position, 1.0);
}
\0";

        const FS_SRC: &[u8] = b"
#version 330

uniform vec3 color;

out vec4 FragColor;

void main() {
    FragColor = vec4(color, 1.0);
}
\0";
        unsafe {
            let vs = gl.CreateShader(gl::VERTEX_SHADER);
            gl.ShaderSource(vs, 1, [VS_SRC.as_ptr() as *const _].as_ptr(), std::ptr::null());
            gl.CompileShader(vs);

            let fs = gl.CreateShader(gl::FRAGMENT_SHADER);
            gl.ShaderSource(fs, 1, [FS_SRC.as_ptr() as *const _].as_ptr(), std::ptr::null());
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
                let cname = std::ffi::CString::new("color").expect("CString::new failed");
                self.loc_color          = gl.GetUniformLocation(self.program, cname.as_ptr());
            }
        }

    }

    pub fn add_element(
        &mut self,
        gl: &gl::Gl,
        mode: gl::types::GLenum,
        elem_vtx: &Vec<gl::types::GLuint>,
        color: [f32;3]) {
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
            self.ebos.push(
                ElementBufferObject {
                    mode: mode, elem_size: elem_vtx.len(), ebo: ebo0, color:color });
        }
    }

    pub fn update_vertex(
        &mut self,
        gl: &gl::Gl,
        vtx_xyz: &Vec<f32>,
        ndim: i32) {
        self.ndim = ndim;
        self.num_point = vtx_xyz.len() as i32 / self.ndim;
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
            for ebo in &self.ebos {
                gl.Uniform3f(self.loc_color, ebo.color[0], ebo.color[1], ebo.color[2]);
                gl.UniformMatrix4fv(self.loc_mat_modelview, 1, gl::FALSE, mat_modelview.as_ptr());
                gl.UniformMatrix4fv(self.loc_mat_projection, 1, gl::FALSE, mp1.as_ptr());
                gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo.ebo);
                gl.DrawElements(ebo.mode,
                                ebo.elem_size as i32,
                                gl::UNSIGNED_INT,
                                std::ptr::null());
            }
        }
    }

    pub fn draw_points(
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
            gl.Uniform3f(self.loc_color, 0., 0., 0.);
            gl.UniformMatrix4fv(self.loc_mat_modelview, 1, gl::FALSE, mat_modelview.as_ptr());
            gl.UniformMatrix4fv(self.loc_mat_projection, 1, gl::FALSE, mp1.as_ptr());
            gl.DrawArrays(gl::POINTS, 0, self.num_point);
        }
    }
}