WITH lamination AS (
SELECT
    SPLIT_PART(part.prt_no, '-', 1) AS material_code,
    UPPER(TO_ASCII(MIN(CASE
        WHEN TRIM(part.prt_desc2) <> '' AND TRIM(part.prt_desc3) <> '' THEN
            TRIM(part.prt_desc2) || ' + ' || TRIM(part.prt_desc3)
        WHEN TRIM(part.prt_desc2) = '' AND TRIM(part.prt_desc3) <> '' THEN
            'BRUT' || ' + ' || TRIM(part.prt_desc3)
        WHEN TRIM(part.prt_desc2) <> '' AND TRIM(part.prt_desc3) = '' THEN
            TRIM(part.prt_desc2)
        ELSE 'BRUT'
    END), 'LATIN1')) AS material_color

FROM
    part
    INNER JOIN part_group
            ON part.pgr_id = part_group.pgr_id

WHERE
    part_group.pgr_no IN ('601', '603', '605', '750', '751')
    
GROUP BY
    material_code
),

lamination_count AS (
SELECT
    SPLIT_PART(part.prt_no, '-', 1) AS material_code,
    MIN((TRIM(part.prt_desc2) <> '')::INT + 
        (TRIM(part.prt_desc3) <> '')::INT) AS count
    
FROM
    part
    INNER JOIN part_group
            ON part.pgr_id = part_group.pgr_id
    
WHERE
    part_group.pgr_no IN ('601', '602', '603', '605', '750', '751')
    
GROUP BY
    material_code
)
  
SELECT
    project.prj_no AS project_number,
    UPPER(TRIM(TO_ASCII(SPLIT_PART(part.prt_no, '/', 1), 'LATIN1'))) AS part_number,
    UPPER(TRIM(TO_ASCII(part.prt_desc1, 'LATIN1'))) AS part_description,
    project.prj_req_qty::INT AS req_quantity,
    COALESCE(SUBSTRING(part.prt_desc1 FROM '\"(.+)\"'), '') AS lettre,
    COALESCE(NULLIF(TRIM(part.prt_dsgn_no), ''), REPLACE(UPPER(TRIM(TO_ASCII(part.prt_no, 'LATIN1'))), '-', '')) AS cnc_program,
    REPLACE(UPPER(TRIM(TO_ASCII(part.prt_no, 'LATIN1'))), '-', '') AS edge_program,
    UPPER(TRIM(part.prt_idx3_3)) AS cnc_machines,
    COALESCE(NULLIF(TRIM(part.prt_idx3_5), ''), '???') AS num_holes,
    ROUND(part.prt_stt_ax1, 2)::TEXT AS length,
    ROUND(part.prt_stt_ax2, 2)::TEXT AS width,
    ROUND(part.prt_stt_ax5, 2)::TEXT AS thickness,
    SPLIT_PART(part.prt_no, '-', 1) AS material_code,
    COALESCE(lamination.material_color, '???') AS material_color,
    COALESCE(lamination_count.count, '0') AS material_faces,
    CASE
        WHEN TRIM(part.prt_idx2_1) = '' THEN project.prj_req_qty
        ELSE TRIM(part.prt_idx2_1)::INT
    END::INT AS parts_per_pallet,
    GENERATE_SERIES(1, COALESCE(CEIL(project.prj_req_qty / NULLIF(TRIM(part.prt_idx2_1), '')::INT), 1)::INT) AS pallet_number,
    COALESCE(CEIL(project.prj_req_qty / NULLIF(part.prt_idx2_1,'')::INT), 1)::INT AS total_pallets,
    UPPER(TRIM(part.prt_idx3_1)) AS machining_time,
    '' AS image_path

FROM       
    project    
    INNER JOIN project_material
            ON project.prt_no = project_material.prt_no
    INNER JOIN part
            ON project.prt_id = part.prt_id
    INNER JOIN part_group 
            ON part.pgr_id = part_group.pgr_id
    LEFT  JOIN lamination 
            ON SPLIT_PART(part.prt_no, '-', 1) = lamination.material_code
    LEFT  JOIN lamination_count
            ON SPLIT_PART(part.prt_no, '-', 1) = lamination_count.material_code

WHERE
        project.prj_source_no = $1
    AND project_material.prj_no = $1
    AND project.prj_req_qty <> 0
    AND part_group.pgr_no IN ('760', '770')
    AND part.prt_desc1 NOT LIKE 'FOAM%'
    AND part.prt_no NOT LIKE '040-ANTIT%'
    AND part.prt_no <> '040-0057'
    AND part.prt_no <> '040-0073'

ORDER BY   
    part_number,
    pallet_number
