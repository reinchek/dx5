for n in {1..1000}; do
    printf -v n "%02d" "$n"

    src="0x00.template.yaml"
    dest="./benchmark/0x${n}.yaml"

    cp "$src" "$dest"
    sed -i "s/{ID}/0x${n}/g" "$dest"

    echo "Creato $dest"
done
