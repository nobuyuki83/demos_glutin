#[test]
fn laplacian() {
    let (tri2vtx, vtx2xyz) = del_msh::io_obj::load_tri_mesh(
        "asset/bunny_11k.obj", Some(1.5));

    let lap = {
        let mut lap = del_ls::sparse_square::Matrix::new();
        let vtx2vtx = del_msh::topology_uniform::psup2(
            &tri2vtx, 3, vtx2xyz.len() / 3);
        lap.initialize_as_square_matrix(&vtx2vtx.0, &vtx2vtx.1);
        lap.set_zero();
        del_misc::mesh_laplacian::merge_trimesh3(
            &mut lap,
            &mut vec!(0_usize; 0),
            &tri2vtx, &vtx2xyz);
        lap
    };

    let bil = del_ls::sparse_matrix_multiplication::mult_square_matrices(
        &lap, &lap);

    {
        let ndof = lap.num_blk;
        let mut a_vec = Vec::<f32>::new();
        for i in 0..ndof {
            a_vec.push( (i as f32 * 0.01).sin() );
        }
        let mut b_vec0= vec!(0_f32; ndof);
        del_ls::sparse_square::gemv_for_sparse_matrix(
            &mut b_vec0, 0_f32, 1_f32, &bil, &a_vec);
        println!("{:?}",b_vec0);

        let mut c_vec1= vec!(0_f32; ndof);
        del_ls::sparse_square::gemv_for_sparse_matrix(
            &mut c_vec1, 0_f32, 1_f32, &lap, &a_vec);
        let mut b_vec1= vec!(0_f32; ndof);
        del_ls::sparse_square::gemv_for_sparse_matrix(
            &mut b_vec1, 0_f32, 1_f32, &lap, &c_vec1);
        println!("{:?}",b_vec1);

        let mut d_vec1= vec!(0_f32; ndof);
        del_ls::slice::sub(&mut d_vec1, &b_vec0, &b_vec1);

        let a0 = del_ls::slice::dot(&d_vec1, &d_vec1);
        println!("{}", a0);
    }
}