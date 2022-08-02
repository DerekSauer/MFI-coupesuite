SELECT
    TRIM(part.prt_no) AS part_code,
    UPPER(TRIM(SPLIT_PART(part.prt_no, '-', 1))) AS material_code,
    part.prt_stt_ax1::REAL AS part_length,
    part.prt_stt_ax2::REAL AS part_width,
    (bill_of_materials_mat.bma_budg_qty / model.prt_std_bqty)::INT AS required_quantity,
    CASE
        WHEN (part.prt_idx2_5 <> '' AND part.prt_idx2_5 <> '0')
        THEN CEIL(bill_of_materials_mat.bma_budg_qty / model.prt_std_bqty / part.prt_idx2_5::INT)
        ELSE bill_of_materials_mat.bma_budg_qty / model.prt_std_bqty
    END::INT AS required_quantity,
    0 AS no_projet,
    COALESCE(TRIM(edge_top.prt_no), '') AS code_edge_haut,
    COALESCE(TRIM(TO_ASCII(edge_top.prt_desc2, 'LATIN1')), '') AS desc_edge_haut,
    COALESCE(TRIM(edge_right.prt_no), '') AS code_edge_droite,
    COALESCE(TRIM(TO_ASCII(edge_right.prt_desc2, 'LATIN1')), '') AS desc_edge_droite,
    COALESCE(TRIM(edge_bottom.prt_no), '') AS code_edge_bas,
    COALESCE(TRIM(TO_ASCII(edge_bottom.prt_desc2, 'LATIN1')), '') AS desc_edge_bas,
    COALESCE(TRIM(edge_left.prt_no), '') AS code_edge_gauche,
    COALESCE(TRIM(TO_ASCII(edge_left.prt_desc2, 'LATIN1')), '') AS desc_edge_gauche,
    0 AS product_code,
    TRIM(REPLACE(model.prt_no, '90-', '')) AS product_information,
    TRIM(TO_ASCII(model.prt_desc1, 'LATIN1')) AS product_description,
    TRIM(COALESCE(SUBSTRING(part.prt_desc1 FROM '\"(.+)\"'), '')) AS lettre_piece,
    'S:/MFI/Dessins/' || part.prt_no || '.jpg' AS picture_filename,
    UPPER(TRIM(TO_ASCII(part.prt_desc1, 'LATIN1'))) AS part_description,
    CASE
        WHEN part.prt_stt_ax1 > 1410.0 OR part.prt_stt_ax2 > 1410.0 THEN 'PALETTE-LONG'
        ELSE 'PALETTE-STANDARD'
    END AS type_palette,
    UPPER(TRIM(part.prt_idx3_1)) = 'KANBAN' AS kanban,
    UPPER(TRIM(part.prt_idx3_4)) = 'P' AS painted

FROM
    part AS model
    INNER JOIN bill_of_materials_mat
            ON model.prt_id = bill_of_materials_mat.prt_master_id
    INNER JOIN part
            ON bill_of_materials_mat.prt_id = part.prt_id
    INNER JOIN part_group
            ON part.pgr_id = part_group.pgr_id
    LEFT  JOIN part AS edge_top
            ON UPPER(TRIM(edge_top.prt_no)) = UPPER(TRIM(part.prt_usr_string1))
    LEFT  JOIN part AS edge_right
            ON UPPER(TRIM(edge_right.prt_no)) = UPPER(TRIM(part.prt_usr_string2))
    LEFT  JOIN part AS edge_bottom
            ON UPPER(TRIM(edge_bottom.prt_no)) = UPPER(TRIM(part.prt_usr_string3))
    LEFT  JOIN part AS edge_left
            ON UPPER(TRIM(edge_left.prt_no)) = UPPER(TRIM(part.prt_usr_string4))

WHERE
        UPPER(TRIM(model.prt_no)) = UPPER(TRIM($1))
    AND part_group.pgr_no IN ('760', '770')             -- FABRICATED PARTS ONLY
    AND part.prt_stt_ax1 <> 0.00                        -- NO PARTS NESTED INSIDE OTHER PARTS
    AND part.prt_stt_ax2 <> 0.00                        -- DITTO
    AND UPPER(TRIM(part.prt_desc1)) NOT LIKE 'FOAM%'    -- NO FOAMS IN WRONG PART GROUP
    AND UPPER(TRIM(part.prt_no)) NOT LIKE '040-ANTI%'   -- NO LABELS IN WRONG PART GROUP
    AND part.prt_no <> '040-0057'                       -- NO CARDBOARD IN WRONG PART GROUP
    AND part.prt_no <> '040-0073'                       -- DITTO

ORDER BY
    part.prt_no
