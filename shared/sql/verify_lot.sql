-- RETURNS A ROW IF THE LOT NUMBER PASSED IN IS A VALID PRODUCTION LOT OF A
-- FURNITURE MODEL. RETURNS NO ROWS IF THE LOT NUMBER IS INVALID.
SELECT
    TRIM(UPPER(project.prt_no)) AS sku

FROM
    project
    INNER JOIN part
            ON project.prt_id = part.prt_id
    INNER JOIN part_group
            ON part.pgr_id = part_group.pgr_id

WHERE
        project.prj_no = $1                 -- MODEL LOT NUMBER
    AND project.prj_source_no = -1          -- MASTER LOTS HAVE NO PARENT
    AND part_group.pgr_no IN ('765', '860', '890') -- FINISHED FURNITURE GROUPS

LIMIT 1
