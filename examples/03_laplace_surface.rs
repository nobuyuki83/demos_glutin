use del_gl::gl as gl;

fn main() {
    let (mut viewer, event_loop) = del_gl::glutin::viewer3::Viewer3::open();

    let (vtx_xyz, tri_vtx) = del_msh::primitive::cylinder_closed_end_tri3(
        0.2, 1.8, 16, 32);
    let mut vtx_val = Vec::<f32>::new();
    vtx_val.resize(vtx_xyz.len() / 3, 0.);

    let mut sparse = del_ls::sparse::BlockSparseMatrix::<f32>::new();
    {  // set partten to sparse matrix
        let elsup = del_msh::topology_uniform::elsup(
            &tri_vtx, 3, vtx_xyz.len() / 3);
        let psup = del_msh::topology_uniform::psup(
            &tri_vtx,
            &elsup.0, &elsup.1,
            3,
            vtx_xyz.len() / 3);
        sparse.initialize_as_square_matrix(&psup.0, &psup.1);
    }

    sparse.set_zero();
    let mut merge_buffer = Vec::<usize>::new();
    for it in 0..tri_vtx.len() / 3 {
        let i0 = tri_vtx[it * 3 + 0];
        let i1 = tri_vtx[it * 3 + 1];
        let i2 = tri_vtx[it * 3 + 2];
        let p0 = &vtx_xyz[(i0 * 3 + 0)..(i0 * 3 + 3)];
        let p1 = &vtx_xyz[(i1 * 3 + 0)..(i1 * 3 + 3)];
        let p2 = &vtx_xyz[(i2 * 3 + 0)..(i2 * 3 + 3)];
        let cots = del_geo::geo_tri::cot3(p0, p1, p2);
        let emat: [f32; 9] = [
            cots[1] + cots[2], -cots[2], -cots[1],
            -cots[2], cots[2] + cots[0], -cots[0],
            -cots[1], -cots[0], cots[0] + cots[1]];
        sparse.merge(
            &[i0, i1, i2], &[i0, i1, i2], &emat,
            &mut merge_buffer);
    }
    drop(merge_buffer);

    let mut r_vec = Vec::<f32>::new();
    r_vec.resize(vtx_xyz.len() / 3, 0.);

    {
        let penalty = 1.0e+3;
        for iv in 0..vtx_xyz.len() / 3 {
            let y = vtx_xyz[iv * 3 + 1];
            if y < -0.89 {
                r_vec[iv] = penalty * 0.;
                sparse.val_dia[iv] += penalty;
            } else if y > 0.89 {
                r_vec[iv] = penalty * 1.;
                sparse.val_dia[iv] += penalty;
            }
        }
    }

    {
        let mut ap_vec = Vec::<f32>::new();
        let mut p_vec = Vec::<f32>::new();
        let conv = del_ls::solver_sparse::solve_cg(
            &mut r_vec, &mut vtx_val,
            &mut ap_vec, &mut p_vec,
            1.0e-5, 100,
            &sparse);
        println!("number of iteration: {}", conv.len());
    }

    let mut drawer = del_gl::drawer_meshposcolor::DrawerMeshPosColor::new();
    {
        drawer.color_map = vec![
            [0.0, 0.0, 0.0], // 0
            [0.5, 0.0, 0.0], // 0.2
            [1.0, 0.0, 0.0], // 0.4
            [1.0, 0.5, 0.0], // 0.6
            [1.0, 1.0, 0.0], // 0.8
            [1.0, 1.0, 1.0]]; // 1.0
        drawer.compile_shader(&viewer.gl);
        drawer.update_vertex(&viewer.gl, &vtx_xyz, 3);
        drawer.update_value(&viewer.gl, &vtx_val);
        drawer.add_element(&viewer.gl, gl::TRIANGLES, &tri_vtx);
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
