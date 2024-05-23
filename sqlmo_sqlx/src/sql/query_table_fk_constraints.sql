
SELECT conname ,
  pg_catalog.pg_get_constraintdef(r.oid, true) as definition
FROM pg_catalog.pg_constraint r
WHERE r.conrelid =
$1::regclass
AND r.contype = 'f' ORDER BY 1

