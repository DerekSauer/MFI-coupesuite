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
    part_group.pgr_no IN ('601', '603', '605', '750', '751', '752', '753', '754')

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
    part_group.pgr_no IN ('601', '602', '603', '605', '750', '751', '752', '753', '754')

GROUP BY
    material_code
),

edgebands as (
    with part_edgebands as (
        select
            part.prt_id,
            unnest(array_remove(array[
                trim(part.prt_usr_string1),
                trim(part.prt_usr_string2),
                trim(part.prt_usr_string3),
                trim(part.prt_usr_string4)
            ], '')) as edgeband

        from
            part
    )

    select
        part_edgebands.prt_id,
        array_agg(distinct part_edgebands.edgeband) as unique_edgebands

    from
        part_edgebands

    group by
        part_edgebands.prt_id
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
    '' AS image_path,
    coalesce(edge1.prt_no, '') as edge1_code,
    coalesce(edge1.prt_desc1, '') as edge1_description,
    coalesce(edge1.prt_stt_ax5, 0.0)::float as edge1_thickness,
    coalesce(edge2.prt_no, '') as edge2_code,
    coalesce(edge2.prt_desc1, '') as edge2_description,
    coalesce(edge2.prt_stt_ax5, 0.0)::float as edge2_thickness,
    coalesce(edge3.prt_no, '') as edge3_code,
    coalesce(edge3.prt_desc1, '') as edge3_description,
    coalesce(edge3.prt_stt_ax5, 0.0)::float as edge3_thickness,
    coalesce(edge4.prt_no, '') as edge4_code,
    coalesce(edge4.prt_desc1, '') as edge4_description,
    coalesce(edge4.prt_stt_ax5, 0.0)::float as edge4_thickness

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
    LEFT  JOIN edgebands
            ON part.prt_id = edgebands.prt_id
    LEFT  JOIN part as edge1
            ON edgebands.unique_edgebands[1] = trim(edge1.prt_no)
    LEFT  JOIN part as edge2
            ON edgebands.unique_edgebands[2] = trim(edge2.prt_no)
    LEFT  JOIN part as edge3
            ON edgebands.unique_edgebands[3] = trim(edge3.prt_no)
    LEFT  JOIN part as edge4
            ON edgebands.unique_edgebands[4] = trim(edge4.prt_no)

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
