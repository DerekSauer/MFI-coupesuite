-- RETURNS A ROW IF THE LOT NUMBER PASSED IN IS A VALID PRODUCTION LOT OF A
-- FURNITURE MODEL. RETURNS NO ROWS IF THE LOT NUMBER IS INVALID.
WITH painted_parts AS (
SELECT
    project.prj_id,
    count(*)

FROM
    project
    INNER JOIN project_material
            ON project.prj_id = project_material.prj_id
    INNER JOIN part
            ON project_material.prt_id = part.prt_id

WHERE
    UPPER(TRIM(part.prt_idx3_4)) = 'P'

GROUP BY
    project.prj_id
)

SELECT
    TRIM(UPPER(TO_ASCII(REPLACE(project.prt_no, '90-', ''), 'LATIN1'))) AS sku,
    TRIM(UPPER(TO_ASCII(project.prj_name, 'LATIN1'))) AS description,
    project.prj_req_qty::INT AS quantity,
    part_group.pgr_no = '765' AS kanban,
    COALESCE(painted_parts.count, 0) > 0 AS painted_parts,
    TRIM(UPPER(TO_ASCII(part.prt_desc3, 'LATIN1'))) AS packing_instructions

FROM
    project
    INNER JOIN part
            ON project.prt_id = part.prt_id
    INNER JOIN part_group
            ON part.pgr_id = part_group.pgr_id
    LEFT  JOIN painted_parts
            ON project.prj_id = painted_parts.prj_id

WHERE
        project.prj_no = $1                         -- MODEL LOT NUMBER
    AND project.prj_source_no = -1                  -- MASTER LOTS HAVE NO PARENT
    AND part_group.pgr_no IN ('765', '860', '890')  -- FINISHED FURNITURE GROUPS

LIMIT 1
