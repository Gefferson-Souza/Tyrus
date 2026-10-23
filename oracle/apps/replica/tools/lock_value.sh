# Lê um campo de "reference" em oracle.lock; exige LOCK definido pelo script que o inclui (source).
lock_value() { python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))["reference"][sys.argv[2]])' "$LOCK" "$1"; }
