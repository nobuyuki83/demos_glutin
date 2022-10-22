use crate::gl;

pub struct DrawerArrayPosColor {
    pub program: gl::types::GLuint,
    pub mode: gl::types::GLenum,
}

impl DrawerArrayPosColor {
    pub fn compile_shader(&mut self, gl: &gl::Gl) {
        const VS_SRC: &[u8] = b"
#version 100
precision mediump float;

attribute vec2 position;
attribute vec3 color;

varying vec3 v_color;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_color = color;
}
\0";

        const FS_SRC: &[u8] = b"
#version 100
precision mediump float;

varying vec3 v_color;

void main() {
    gl_FragColor = vec4(v_color, 1.0);
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
        }
    }

    pub fn initialize(
        &self,
        gl: &gl::Gl,
        vtx_xyz: &Vec<f32>) {
        unsafe {
            let mut vb = std::mem::zeroed();
            gl.GenBuffers(1, &mut vb);
            gl.BindBuffer(gl::ARRAY_BUFFER, vb);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (vtx_xyz.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vtx_xyz.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            if gl.BindVertexArray.is_loaded() {
                let mut vao = std::mem::zeroed();
                gl.GenVertexArrays(1, &mut vao);
                gl.BindVertexArray(vao);
            }

            let pos_attrib = gl.GetAttribLocation(self.program, b"position\0".as_ptr() as *const _);
            let color_attrib = gl.GetAttribLocation(self.program, b"color\0".as_ptr() as *const _);
            gl.VertexAttribPointer(
                pos_attrib as gl::types::GLuint,
                2,
                gl::FLOAT,
                0,
                5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            gl.VertexAttribPointer(
                color_attrib as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (2 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            gl.EnableVertexAttribArray(color_attrib as gl::types::GLuint);
        }
    }

    pub fn draw_frame(
        &self,
        gl: &gl::Gl) {
        unsafe {
            gl.UseProgram(self.program);
            gl.DrawArrays(self.mode, 0, 3);
        }
    }
}
