use del_gl::gl as gl;

fn main() {
    let (tri2vtx, vtx2xyz) = del_msh::io_obj::load_tri_mesh(
        "asset/bunny_11k.obj", Some(1.5));

    let mut ls_laplace = del_ls::linearsystem::Solver::new();
    {  // set pattern to sparse matrix
        let vtx2tri = del_msh::topology_uniform::elsup(
            &tri2vtx, 3, vtx2xyz.len() / 3);
        let vtx2vtx = del_msh::topology_uniform::psup(
            &tri2vtx,
            &vtx2tri.0, &vtx2tri.1,
            3,
            vtx2xyz.len() / 3);
        ls_laplace.initialize(&vtx2vtx.0, &vtx2vtx.1);
    }

    // sparse.set_zero();
    ls_laplace.begin_mearge();
    for it in 0..tri2vtx.len() / 3 {
        let i0 = tri2vtx[it * 3 + 0];
        let i1 = tri2vtx[it * 3 + 1];
        let i2 = tri2vtx[it * 3 + 2];
        let cots = del_geo::tri::cot3(
            &vtx2xyz[(i0 * 3 + 0)..(i0 * 3 + 3)],
            &vtx2xyz[(i1 * 3 + 0)..(i1 * 3 + 3)],
            &vtx2xyz[(i2 * 3 + 0)..(i2 * 3 + 3)]);
        let emat: [f32; 9] = [
            cots[1] + cots[2], -cots[2], -cots[1],
            -cots[2], cots[2] + cots[0], -cots[0],
            -cots[1], -cots[0], cots[0] + cots[1]];
        ls_laplace.sparse.merge(
            &[i0, i1, i2], &[i0, i1, i2], &emat,
            &mut ls_laplace.merge_buffer);
    }

    let mut ls_heat = ls_laplace.clone();

    {
        let vtx2area = del_misc::mesh_property::area_par_vertex_in_triangles_mesh(&tri2vtx, &vtx2xyz);
        let h0 = del_misc::mesh_property::mean_edge_length_triangles_mesh(&tri2vtx, &vtx2xyz);
        let m0 = 100_f32;
        let t = m0 * h0 * h0;
        for iv in 0..ls_heat.sparse.val_dia.len() {
            ls_heat.sparse.val_dia[iv] = ls_heat.sparse.val_dia[iv] * t + vtx2area[iv];
        }
        ls_heat.sparse.val_crs.iter_mut().for_each(|v| *v = (*v) * t );
        ls_heat.r_vec[0] = 1.0; // the integrated amount of heat is 1
    }

    ls_heat.conv_ratio_tol = 1.0e-5;
    ls_heat.max_num_iteration = 1000;
    ls_heat.solve_cg();
    println!("num_iteration for heat: {}", ls_heat.conv.len());
    let vtx2heat = ls_heat.u_vec.clone();

    let tri2dir = {
        let mut tri2dir = vec!(0_f32; tri2vtx.len());
        del_misc::heat_distance::direction_gradient_on_surface(
            &mut tri2dir,
            &tri2vtx, &vtx2xyz, &vtx2heat);
        tri2dir
    };

    {
        ls_laplace.r_vec.iter_mut().for_each(|v| *v = 0_f32);
        del_misc::heat_distance::divergence_on_trimesh3(
            &mut ls_laplace.r_vec,
            &tri2vtx, &vtx2xyz, &tri2dir);
        ls_laplace.sparse.val_dia[0] += 10000_f32;
    }
    ls_laplace.conv_ratio_tol = 1.0e-5;
    ls_laplace.max_num_iteration = 1000;
    ls_laplace.solve_cg();
    println!("num_iteration for distance: {}", ls_laplace.conv.len());
    let vtx2dist = ls_laplace.u_vec.clone();

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
        drawer_color.update_value(&viewer.gl, &vtx2dist);
        drawer_color.add_element(&viewer.gl, gl::TRIANGLES, &tri2vtx);
    }

    let mut drawer_edge = del_gl::array::Drawer {
        program: 0,
        mode: gl::LINES,
        elem_size: tri2vtx.len() * 2,
        vao: 0,
        loc_color: -1,
        loc_mat_projection: -1,
        loc_mat_modelview: -1,
        color: [0., 0., 1.],
    };
    {
        let mut tri2xyz = vec!(0_f32; tri2vtx.len() * 2);
        for i_tri in 0..tri2vtx.len() / 3 {
            let i0 = tri2vtx[i_tri * 3 + 0];
            let i1 = tri2vtx[i_tri * 3 + 1];
            let i2 = tri2vtx[i_tri * 3 + 2];
            let p0 = &vtx2xyz[i0 * 3..i0 * 3 + 3];
            let p1 = &vtx2xyz[i1 * 3..i1 * 3 + 3];
            let p2 = &vtx2xyz[i2 * 3..i2 * 3 + 3];
            tri2xyz[i_tri * 6 + 0] = (p0[0] + p1[0] + p2[0]) / 3_f32;
            tri2xyz[i_tri * 6 + 1] = (p0[1] + p1[1] + p2[1]) / 3_f32;
            tri2xyz[i_tri * 6 + 2] = (p0[2] + p1[2] + p2[2]) / 3_f32;
            let scale = 0.02;
            tri2xyz[i_tri * 6 + 3] = tri2xyz[i_tri * 6 + 0] + tri2dir[i_tri * 3 + 0] * scale;
            tri2xyz[i_tri * 6 + 4] = tri2xyz[i_tri * 6 + 1] + tri2dir[i_tri * 3 + 1] * scale;
            tri2xyz[i_tri * 6 + 5] = tri2xyz[i_tri * 6 + 2] + tri2dir[i_tri * 3 + 2] * scale;
        }
        drawer_edge.compile_shader(&viewer.gl);
        drawer_edge.initialize(&viewer.gl, &tri2xyz);
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
            drawer_color.val_max = 1.0;
            drawer_color.draw(
                &viewer.gl,
                mat_modelview.as_slice(), // nalgebra is column major same as OpenGL
                mat_projection.as_slice()); // nalgebra is column major same as OpenGL
            drawer_edge.draw_frame(&viewer.gl,
                                   mat_modelview.as_slice(),
                                   mat_projection.as_slice());
            viewer.windowed_context.swap_buffers().unwrap();
        }
    };
    event_loop.run(event_handle_closure);
}
