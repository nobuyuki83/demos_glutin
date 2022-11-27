use del_gl::gl as gl;

fn main() {
    let (tri2vtx, vtx2xyz) = del_msh::io_obj::load_tri_mesh(
        "asset/bunny_1k.obj", Some(1.5));

    let mut ls = {  // set pattern to sparse matrix
        let vtx2vtx = del_msh::vtx2vtx::from_uniform_mesh2(
            &tri2vtx, 3, vtx2xyz.len() / 3);
        let mut m = del_ls::sparse_square::Matrix::<f64>::new();
        m.symbolic_initialization(&vtx2vtx.0, &vtx2vtx.1);
        m.set_zero();
        del_misc::mesh_laplacian::merge_trimesh3(
            &mut m,
            &mut vec!(),
            &tri2vtx, &vtx2xyz);
        let m = del_ls::sparse_matrix_multiplication::mult_square_matrices(
            &m, &m);
        let mut ls = del_ls::linearsystem::Solver::<f64>::new();
        ls.sparse = m;
        ls.r_vec = vec!(0_f64; ls.sparse.num_blk);
        {
            let penalty = 1.0e+2;
            ls.r_vec[10] += penalty;
            ls.sparse.row2val[10] += penalty;
            //
            let ivtx0: usize = 100;
            ls.sparse.row2val[ivtx0] += penalty;
        }
        ls.ilu.initialize_iluk(&ls.sparse, 3);
        // ls.ilu.initialize_full(ls.sparse.num_blk);
        println!("{}", ls.ilu.idx2col.len());
        del_ls::sparse_ilu::copy_value(&mut ls.ilu, &ls.sparse);
        del_ls::sparse_ilu::decompose(&mut ls.ilu);
        ls
    };
    // -------------
    ls.conv_ratio_tol = 1.0e-5;
    ls.max_num_iteration = 5000;
    ls.solve_pcg();
    println!("num_iteration for heat: {}", ls.conv.len());
    println!("{:?}", ls.conv.last());
    let vtx2heat = ls.u_vec.clone();
    let vtx2heat = vtx2heat.iter().map(|v| *v as f32).collect();
    //println!("{:?}",vtx2heat);

    let (mut viewer, event_loop) = del_gl::glutin::viewer3::Viewer3::open();

    let mut drawer_color = del_gl::mesh_colormap::Drawer::new();
    {
        drawer_color.color_map = vec![
            [0.0, 0.0, 0.0], // 0
            [0.5, 0.0, 0.0], // 0.2
            [1.0, 0.0, 0.0], // 0.4
            [1.0, 0.5, 0.0], // 0.6
            [1.0, 1.0, 0.0], // 0.8
            [1.0, 1.0, 1.0]]; // 1.0
        drawer_color.compile_shader(&viewer.gl);
        drawer_color.update_vertex(&viewer.gl, &vtx2xyz, 3);
        drawer_color.update_value(&viewer.gl, &vtx2heat);
        drawer_color.add_element(&viewer.gl, gl::TRIANGLES, &tri2vtx);
    }

    // this closure captures drawer, viewer and 'move' them. drawer and viewer cannot be usable anymore
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
            drawer_color.val_max = 1.0;
            drawer_color.val_min = 0.0;
            drawer_color.draw(
                &viewer.gl,
                mat_modelview.as_slice(), // nalgebra is column major same as OpenGL
                mat_projection.as_slice()); // nalgebra is column major same as OpenGL
            viewer.windowed_context.swap_buffers().unwrap();
        }
    };
    event_loop.run(event_handle_closure);
}
