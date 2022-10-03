WITH common_skus AS (
SELECT
    bill_of_materials_mat.prt_id AS prt_id,
    ARRAY_AGG(UPPER(TRIM(TO_ASCII(REPLACE(model.prt_no, '90-', ''), 'LATIN1')))) AS sku

FROM
    bill_of_materials_mat
    INNER JOIN part AS model
            ON bill_of_materials_mat.prt_master_id = model.prt_id
    INNER JOIN part
            ON bill_of_materials_mat.prt_id = part.prt_id

WHERE
        model.prt_active
    AND part.prt_active
    AND model.prt_no NOT LIKE 'OLD%'
    AND model.prt_no NOT LIKE '%T'
    AND model.prt_no NOT LIKE '%Y'

GROUP BY
    part.prt_creation_dt,
    bill_of_materials_mat.prt_id

ORDER BY
    part.prt_creation_dt DESC
)

SELECT
    COALESCE(UPPER(TRIM(SUBSTRING(part.prt_desc1 FROM '\"(.+)\"'))), '') AS part_letter,
    project.prj_req_qty::INT AS qty_to_ship,
    project.prj_no AS part_lot_number,
    UPPER(TRIM(TO_ASCII(part.prt_no, 'LATIN1'))) AS part_number,
    UPPER(TRIM(TO_ASCII(part.prt_desc1, 'LATIN1'))) AS part_description,
    ARRAY_TO_STRING(common_skus.sku[1:10], ', ') AS common_skus,
    UPPER(TRIM(part.prt_idx3_1)) AS machining_time

FROM
    project
    INNER JOIN part
            ON project.prt_id = part.prt_id
    INNER JOIN part_group
            ON part.pgr_id = part_group.pgr_id
    LEFT  JOIN common_skus
            ON part.prt_id = common_skus.prt_id

WHERE
        project.prj_source_no = $1
    AND part_group.pgr_no IN ('760', '770')
    AND part.prt_desc1 NOT LIKE 'FOAM%'
    AND part.prt_no NOT LIKE '040-ANTIT%'
    AND part.prt_no <> '040-0057'
    AND part.prt_no <> '040-0073'

ORDER BY
    part_letter
