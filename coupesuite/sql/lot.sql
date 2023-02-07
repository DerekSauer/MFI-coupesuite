SELECT
    STRING_AGG(project.prj_no::TEXT, ',' ORDER BY project.prj_no) AS lot_numbers

FROM
    project
    INNER JOIN part
            ON project.prt_id = part.prt_id
    INNER JOIN part_group
            ON part.pgr_id = part_group.pgr_id

WHERE
        project.prj_source_no = -1
    AND project.prj_start_dt = CURRENT_DATE
    AND part_group.pgr_no IN ('890', '765')
