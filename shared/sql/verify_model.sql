-- RETURNS A ROW IF THE MODEL NUMBER PASSED IN IS A VALID
-- FURNITURE MODEL. RETURNS NO ROWS IF THE MODEL NUMBER IS INVALID.
WITH painted_parts AS (
SELECT
    model.prt_id,
    count(*)
    
FROM
    part AS model
    INNER JOIN bill_of_materials_mat
            ON model.prt_id = bill_of_materials_mat.prt_master_id
    INNER JOIN part
            ON bill_of_materials_mat.prt_id = part.prt_id
    
WHERE
    UPPER(TRIM(part.prt_idx3_4)) = 'P'
    
GROUP BY
    model.prt_id
)

SELECT
    TRIM(UPPER(TO_ASCII(part.prt_no, 'LATIN1'))) AS sku,
    TRIM(UPPER(TO_ASCII(part.prt_desc1, 'LATIN1'))) AS description,
    0::INT AS quantity,
    part_group.pgr_no = '765' AS kanban,
    COALESCE(painted_parts.count, 0) > 0 AS painted_parts

FROM
    part
    INNER JOIN part_group
            ON part.pgr_id = part_group.pgr_id
    LEFT  JOIN painted_parts
            ON part.prt_id = painted_parts.prt_id

WHERE
        TRIM(UPPER(part.prt_no)) = TRIM(UPPER($1))
    AND part_group.pgr_no IN ('765', '860', '890')

LIMIT 1
