use del_gl::gl as gl;

fn resampling_loops(
    loop_vtx_idx: &mut Vec<usize>,
    loop_vtx: &mut Vec<usize>,
    vtx_xy: &mut Vec<nalgebra::Vector2<f32>>,
    max_edge_length: f32) {
    assert_eq!(vtx_xy.len(), loop_vtx.len());
    let loop_vtx_idx0 = loop_vtx_idx.clone();
    let loop_vtx0 = loop_vtx.clone();
    assert!(loop_vtx0.len() >= 2);
    let nloop = loop_vtx_idx0.len() - 1;
    let mut point_idxs_in_edges = vec!(Vec::<usize>::new(); loop_vtx0.len());

    for iloop in 0..nloop {
        assert!(loop_vtx_idx0[iloop + 1] > loop_vtx_idx0[iloop]);
        let np = loop_vtx_idx0[iloop + 1] - loop_vtx_idx0[iloop];
        for ip in 0..np {
            let iipo0 = loop_vtx_idx0[iloop] + (ip + 0) % np;
            let iipo1 = loop_vtx_idx0[iloop] + (ip + 1) % np;
            assert!(iipo0 < loop_vtx0.len());
            assert!(iipo1 < loop_vtx0.len());
            let ipo0 = loop_vtx0[iipo0];
            let ipo1 = loop_vtx0[iipo1];
            assert!(ipo0 < vtx_xy.len());
            assert!(ipo1 < vtx_xy.len());
            let po0 = vtx_xy[ipo0].clone(); // never use reference here because aVec2 will resize afterward
            let po1 = vtx_xy[ipo1].clone(); // never use reference here because aVec2 will resize afterward
            let nadd = ((po0 - po1).norm() / max_edge_length) as i32;
            if nadd == 0 {
                continue;
            }
            for iadd in 0..nadd {
                let r2 = (iadd + 1) as f32 / (nadd + 1) as f32;
                let v2 = (1. - r2) * po0 + r2 * po1;
                let ipo2 = vtx_xy.len();
                vtx_xy.push(v2);
                assert!(iipo0 < point_idxs_in_edges.len());
                point_idxs_in_edges[iipo0].push(ipo2);
            }
        }
    }
    //
    loop_vtx_idx.resize(nloop + 1, usize::MAX);
    loop_vtx_idx[0] = 0;
    for iloop in 0..nloop {
        let nbar0 = loop_vtx_idx0[iloop + 1] - loop_vtx_idx0[iloop];
        let mut nbar1 = nbar0;
        for ibar in 0..nbar0 {
            let iip_loop = loop_vtx_idx0[iloop] + ibar;
            nbar1 += point_idxs_in_edges[iip_loop].len();
        }
        loop_vtx_idx[iloop + 1] = loop_vtx_idx[iloop] + nbar1;
    }
    // adding new vertices on the outline
    loop_vtx.resize(loop_vtx_idx[nloop], usize::MAX);
    let mut ivtx0 = 0;
    for iloop in 0..nloop {
        for iip_loop in loop_vtx_idx0[iloop]..loop_vtx_idx0[iloop + 1] {
            let ip_loop = loop_vtx0[iip_loop];
            loop_vtx[ivtx0] = ip_loop;
            ivtx0 += 1;
            for iadd in 0..point_idxs_in_edges[ip_loop].len() {
                loop_vtx[ivtx0] = point_idxs_in_edges[iip_loop][iadd];
                ivtx0 += 1;
            }
        }
    }
    assert_eq!(loop_vtx.len(), vtx_xy.len());
    assert_eq!(loop_vtx.len(), ivtx0);
}

fn main() {
    use del_dtri::topology::{
        DynamicTriangle,
        DynamicVertex};
    use del_dtri::mesher2::meshing_single_connected_shape2;
    use del_dtri::mesher2::meshing_inside;
    let mut tri_vtx = Vec::<DynamicTriangle>::new();
    let mut vtx_tri = Vec::<DynamicVertex>::new();
    let mut vtx_xy = Vec::<nalgebra::Vector2<f32>>::new();
    {
        let mut loop_vtx_idx = vec!(0, 4);
        let mut loop_vtx = vec!(0, 1, 2, 3);
        vtx_xy.push(nalgebra::Vector2::<f32>::new(-1.0, -1.0));
        vtx_xy.push(nalgebra::Vector2::<f32>::new(1.0, -1.0));
        vtx_xy.push(nalgebra::Vector2::<f32>::new(1.0, 1.0));
        vtx_xy.push(nalgebra::Vector2::<f32>::new(-1.0, 1.0));
        resampling_loops(
            &mut loop_vtx_idx, &mut loop_vtx, &mut vtx_xy,
            0.12);
        meshing_single_connected_shape2(
            &mut vtx_tri, &mut vtx_xy, &mut tri_vtx,
            &loop_vtx_idx, &loop_vtx);
        let mut vtx_flg = vec!(0; vtx_tri.len());
        let mut tri_flg = vec!(0; tri_vtx.len());
        let nvtx0 = vtx_xy.len();
        meshing_inside(
            &mut vtx_tri, &mut tri_vtx, &mut vtx_xy,
            &mut vtx_flg, &mut tri_flg,
            nvtx0, 0, 0.12);
    }

    let (mut viewer, event_loop) = del_gl::glutin::viewer2::Viewer2::open();

    let mut drawer = del_gl::drawer_meshpos::DrawerMeshPos::new();
    {
        use crate::gl::types::GLuint;
        drawer.compile_shader(&viewer.gl);
        {
            let mut vtx_xyz = Vec::<f32>::new();
            for p in &vtx_xy {
                vtx_xyz.push(p.x);
                vtx_xyz.push(p.y);
            }
            drawer.update_vertex(&viewer.gl, &vtx_xyz, 2);
        }
        let tri_vtx0 = {
            let mut tri_vtx0 = Vec::<usize>::new();
            for t in &tri_vtx {
                tri_vtx0.push(t.v[0]);
                tri_vtx0.push(t.v[1]);
                tri_vtx0.push(t.v[2]);
            }
            tri_vtx0
        };
        {
            let tri_vtx1: Vec<GLuint> = tri_vtx0.iter().map(|i| *i as gl::types::GLuint).collect();
            drawer.add_element(&viewer.gl, gl::TRIANGLES, &tri_vtx1, [1., 0., 0.]);
        }
        {
            let line_vtx: Vec<usize> = del_msh::topology_uniform::mshline(
                &tri_vtx0, 3,
                &[0, 1, 1, 2, 2, 0],
                vtx_xy.len());
            let line_vtx0: Vec<GLuint> = line_vtx.iter().map(|i| *i as gl::types::GLuint).collect();
            drawer.add_element(&viewer.gl, gl::LINES, &line_vtx0, [0., 0., 0.]);
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
            let mat_projection = viewer.nav.projection_matrix(viewer.ui_state.win_width, viewer.ui_state.win_height);
            let mat_modelview = viewer.nav.modelview_matrix();
            drawer.draw(
                &viewer.gl,
                mat_modelview.as_slice(), // nalgebra is column major same as OpenGL
                mat_projection.as_slice()); // nalgebra is column major same as OpenGL
            drawer.draw_points(
                &viewer.gl,
                mat_modelview.as_slice(), // nalgebra is column major same as OpenGL
                mat_projection.as_slice()); // nalgebra is column major same as OpenGL
            viewer.windowed_context.swap_buffers().unwrap();
        }
    };
    event_loop.run(event_handle_closure);
}
