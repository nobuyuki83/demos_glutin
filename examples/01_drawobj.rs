use del_gl::gl as gl;

fn main() {
    let (mut viewer, event_loop) = del_gl::glutin::viewer3::Viewer3::open();

    let mut drawer = del_gl::drawer_meshpos::DrawerMeshPos::new();
    {
        let filename: &str = "asset/bunny_1k.obj";
        let tri_vtx: Vec<usize>;
        let vtx_xyz: Vec<f32> = {
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
        let line_vtx: Vec<usize> = del_msh::topology_uniform::mshline(
            &tri_vtx, 3,
            &[0, 1, 1, 2, 2, 0],
            vtx_xyz.len() / 3);
        use crate::gl::types::GLuint;
        let elem_vtx0: Vec<GLuint> = tri_vtx.iter().map(|i| *i as gl::types::GLuint).collect();
        let mshline0: Vec<GLuint> = line_vtx.iter().map(|i| *i as gl::types::GLuint).collect();
        drawer.compile_shader(&viewer.gl);
        drawer.update_vertex(&viewer.gl, &vtx_xyz, 3);
        drawer.add_element(&viewer.gl, gl::TRIANGLES, &elem_vtx0, [1., 0., 0.]);
        drawer.add_element(&viewer.gl, gl::LINES, &mshline0, [0., 0., 0.]);
    }

    // this clousure captures drawer, viewer and 'move' them. drawer and viewer cannot be usable anymore
    let event_handle_closure = move |event: glutin::event::Event<()>,
                                     _elwt: &glutin::event_loop::EventLoopWindowTarget<()>,
                                     control_flow: &mut glutin::event_loop::ControlFlow| {
        viewer.event_handle(&event);
        use glutin::event_loop::ControlFlow::{Wait, ExitWithCode};
        if viewer.should_close { *control_flow = ExitWithCode(0); } else { *control_flow = Wait; }
        //
        if viewer.should_draw {
            let mat_projection = viewer.nav.projection_matrix(
                viewer.ui_state.win_width, viewer.ui_state.win_height);
            let mat_modelview = viewer.nav.modelview_matrix();
            drawer.draw(
                &viewer.gl,
                mat_modelview.as_slice(), // nalgebra is column major same as OpenGL
                mat_projection.as_slice()); // nalgebra is column major same as OpenGL
            viewer.windowed_context.swap_buffers().unwrap();
        }
    };
    event_loop.run(event_handle_closure);
}
