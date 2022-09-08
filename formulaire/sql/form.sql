SELECT
	TRIM(part.prt_no) AS part_number,
	UPPER(TRIM(TO_ASCII(part.prt_desc1, 'LATIN1'))) AS description,
	project.prj_req_qty::INT AS quantity,
	ROUND(part.prt_stt_ax1, 2)::TEXT AS length,
	ROUND(part.prt_stt_ax2, 2)::TEXT AS width,
	UPPER(TRIM(part.prt_idx3_1)) AS machining_time,
	UPPER(TRIM(part.prt_idx3_2)) AS edging,
	UPPER(TRIM(part.prt_idx3_3)) AS robot,
	UPPER(TRIM(part.prt_idx3_4)) AS manual_edge

FROM
	project
	INNER JOIN part oN project.prt_id = part.prt_id
	INNER JOIN part_group ON part.pgr_id = part_group.pgr_id

WHERE
	project.prj_source_no = $1
    AND part_group.pgr_no IN ('760', '770')

    -- FILTER OUT MISCLASSIFIED COMPONENTS
	AND NOT TRIM(UPPER(part.prt_desc1)) LIKE 'FOAM%'
	AND NOT TRIM(UPPER(part.prt_no)) LIKE '040-ANTI%'
	AND part.prt_no <> '040-0073'
	AND part.prt_no <> '040-0057'

ORDER BY
	part_number
