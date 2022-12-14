use del_gl::gl as gl;

fn main() {
    let (mut viewer, event_loop) = del_gl::glutin::viewer3::Viewer3::open();

    let mut drawer = del_gl::mesh::Drawer::new();
    {
        let tri2vtx_xyz: Vec<usize>;
        let vtx_xyz2xyz: Vec<f32>;
        let tri2vtx_uv: Vec<usize>;
        let vtx_uv2uv: Vec<f32>;
        {
            let mut obj = del_msh::io_obj::WavefrontObj::<f32>::new();
            let filename: &str = "asset/Babi/babi.obj";
            obj.load(filename);
            let num_elem = obj.elem2idx.len() - 1;
            println!("element size: {}", num_elem);
            assert_eq!(obj.elem2idx[num_elem - 1], (num_elem - 1) * 3);  // triangle mesh
            tri2vtx_xyz = obj.idx2vtx_xyz;
            vtx_xyz2xyz = del_misc::nalgebra::msh_misc::centerize_normalize_boundingbox(obj.vtx2xyz, 3);
            vtx_uv2uv = obj.vtx2uv;
            tri2vtx_uv = obj.idx2vtx_uv;
        };
        println!("vertex size: {}", vtx_xyz2xyz.len() / 3);
        println!("uv size: {}", vtx_uv2uv.len() / 2);
        let (num_group, tri2group) = {
            let tri2tri = del_msh::elem2elem::from_uniform_mesh2(
                &tri2vtx_uv, 3,
                &[0, 2, 4, 6], &[1, 2, 2, 0, 0, 1],
                vtx_uv2uv.len() / 2);
            let (num_group, tri2group) = del_msh::group::make_group_elem(
                &tri2vtx_uv, 3, &tri2tri);
            (num_group, tri2group)
        };
        println!("num_group: {}", num_group);
        drawer.compile_shader(&viewer.gl);
        drawer.update_vertex(&viewer.gl, &vtx_xyz2xyz, 3);
        for i_group in 0..num_group {
            let mut tri2vtx0 = Vec::<usize>::new();
            for i_elem in 0..tri2group.len() {
                if tri2group[i_elem] == i_group {
                    tri2vtx0.push(tri2vtx_xyz[i_elem * 3 + 0]);
                    tri2vtx0.push(tri2vtx_xyz[i_elem * 3 + 1]);
                    tri2vtx0.push(tri2vtx_xyz[i_elem * 3 + 2]);
                }
            }
            let r = (i_group % 3 + 1) as f32 / 3 as f32;
            let g = (i_group % 4 + 1) as f32 / 4 as f32;
            let b = (i_group % 5 + 1) as f32 / 5 as f32;
            drawer.add_element(&viewer.gl, gl::TRIANGLES, &tri2vtx0, [r, g, b]);
        }
        {
            let line2vtx_xyz: Vec<usize> = del_msh::line2vtx::from_sepecific_edges_of_uniform_mesh(
                &tri2vtx_xyz, 3,
                &[0, 1, 1, 2, 2, 0],
                vtx_xyz2xyz.len() / 3);
            drawer.add_element(&viewer.gl, gl::LINES, &line2vtx_xyz, [0., 0., 0.]);
        }
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
