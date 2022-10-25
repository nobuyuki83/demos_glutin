use del_gl::gl as gl;

struct ScaleRotTrans {
    pub s: f32,
    pub quaternion: nalgebra::UnitQuaternion<f32>,
    pub translation: nalgebra::geometry::Translation3<f32>,
}

impl ScaleRotTrans {
    pub fn new() -> Self {
        ScaleRotTrans {
            s: 1_f32,
            quaternion: nalgebra::UnitQuaternion::<f32>::identity(),
            translation: nalgebra::geometry::Translation3::<f32>::new(0., 0., 0.),
        }
    }

    pub fn to_homogenous(&self) -> nalgebra::Matrix4<f32> {
        let ms = nalgebra::geometry::Scale3::new(
            self.s, self.s, self.s).to_homogeneous();
        let mt = nalgebra::geometry::Translation3::new(
            self.translation.x, self.translation.y, self.translation.z).to_homogeneous();
        let mr = self.quaternion.to_homogeneous();
        mt * mr * ms
    }
}

fn main() {
    let (mut viewer, event_loop) = del_gl::glutin_viewer3::GlutinViewer3::open();

    let tri_vtx: Vec<usize>;
    let vtx_xyz: Vec<f32> = {
        let filename: &str = "asset/bunny_1k.obj";
        let mut obj = del_msh::io_obj::WavefrontObj::<f32>::new();
        obj.load(filename);
        println!("vertex size: {}", obj.vtx_xyz.len() / 3);
        println!("element size: {}", obj.elem_vtx_index.len() - 1);
        tri_vtx = obj.elem_vtx_xyz.iter().map(|i| *i as usize).collect();
        let mut vtx_xyz0 = obj.vtx_xyz.clone();
        for iv in 0..vtx_xyz0.len() / 3 {
            vtx_xyz0[iv * 3 + 0] *= 0.03;
            vtx_xyz0[iv * 3 + 1] *= 0.03;
            vtx_xyz0[iv * 3 + 2] *= 0.03;
        }
        vtx_xyz0
    };

    let mut drawer_mesh = del_gl::drawer_meshpos::DrawerMeshPos::new();
    {
        let line_vtx: Vec<usize> = del_msh::topology_uniform::mshline(
            &tri_vtx, 3,
            &[0, 1, 1, 2, 2, 0],
            vtx_xyz.len() / 3);
        use crate::gl::types::GLuint;
        let elem_vtx0: Vec<GLuint> = tri_vtx.iter().map(|i| *i as gl::types::GLuint).collect();
        let mshline0: Vec<GLuint> = line_vtx.iter().map(|i| *i as gl::types::GLuint).collect();
        drawer_mesh.compile_shader(&viewer.gl);
        drawer_mesh.update_vertex(&viewer.gl, &vtx_xyz, 3);
        drawer_mesh.add_element(&viewer.gl, gl::TRIANGLES, &elem_vtx0, [1., 1., 1.]);
        drawer_mesh.add_element(&viewer.gl, gl::LINES, &mshline0, [0., 0., 0.]);
    }

    let mut drawer_sphere = del_gl::drawer_meshpos::DrawerMeshPos::new();
    {
        let sphere_meshtri3 = del_msh::primitive::sphere_tri3::<f32>(
            1., 32, 32);
        use crate::gl::types::GLuint;
        let tri_vtx0: Vec<GLuint> = sphere_meshtri3.1.iter().map(|i| *i as gl::types::GLuint).collect();
        drawer_sphere.compile_shader(&viewer.gl);
        drawer_sphere.update_vertex(&viewer.gl, &(sphere_meshtri3.0), 3);
        drawer_sphere.add_element(&viewer.gl, gl::TRIANGLES, &tri_vtx0, [1., 0., 0.]);
    }
    let mut transform_sphere = ScaleRotTrans::new();
    transform_sphere.s = 0.03;
    transform_sphere.translation = nalgebra::geometry::Translation3::new(
        vtx_xyz[0], vtx_xyz[1], vtx_xyz[2]);

    // this clousure captures drawer, viewer and 'move' them. drawer and viewer cannot be usable anymore
    let event_handle_closure = move |event: glutin::event::Event<()>,
                                     _elwt: &glutin::event_loop::EventLoopWindowTarget<()>,
                                     control_flow: &mut glutin::event_loop::ControlFlow| {
        viewer.event_handle(&event);
        use glutin::event_loop::ControlFlow::{Wait, ExitWithCode};

        if viewer.should_close { *control_flow = ExitWithCode(0); } else { *control_flow = Wait; }
        if viewer.is_left_btn_down_not_for_view_ctrl {
            let (ray_org, ray_dir) = viewer.nav.picking_ray(
                viewer.ui_state.win_width, viewer.ui_state.win_height,
                viewer.ui_state.cursor_x, viewer.ui_state.cursor_y);
            let res = del_misc::srch_bruteforce::intersection_meshtri3(
                &ray_org.as_slice(), &ray_dir.as_slice(), &vtx_xyz, &tri_vtx);
            match res {
                None => { println!("no hit!") }
                Some(postri) => {
                    let pos = postri.0;
                    transform_sphere.translation = nalgebra::geometry::Translation3::new(
                        pos[0], pos[1], pos[2]);
                }
            }
            viewer.windowed_context.window().request_redraw();
        }
        if viewer.should_draw {
            let mat_projection = viewer.nav.projection_matrix(
                viewer.ui_state.win_width, viewer.ui_state.win_height );
            let mat_modelview = viewer.nav.modelview_matrix();
            drawer_mesh.draw(
                &viewer.gl,
                mat_modelview.as_slice(), // nalgebra is column major same as OpenGL
                mat_projection.as_slice()); // nalgebra is column major same as OpenGL
            let mat_mvo = mat_modelview * transform_sphere.to_homogenous();
            drawer_sphere.draw(
                &viewer.gl,
                mat_mvo.as_slice(), // nalgebra is column major same as OpenGL
                mat_projection.as_slice()); // nalgebra is column major same as OpenGL
            viewer.windowed_context.swap_buffers().unwrap();
        }
    };
    event_loop.run(event_handle_closure);
}