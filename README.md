# ypbank

Сборка проекта осуществляется с использованием команды 

```
cargo build
```

Документация открывается с использованием команды

```
cargo doc --open
```

Все библиотеки находятся в директории libs

**ypbank_transaction** -- библиотека, предназначенная для чтениия, парсинга и анализа данных файлов типа txt, bin, csv

## Улитилиты

**./target/debug/ypbank_comparer** -- утилита для сравнения файлов. Параметры могут передваться в различном порядке, лишь бы была хотя бы пара параметров `--formatN` и `--fileN`, где N -- неотрицательное целое число. 

* `--formatN` -- формат файла, значения: csv, txt, bin
* `--fileN` -- абсолютный/относительный путь до файла

Пример запуска

```
./target/debug/ypbank_comparer --file1 samples/records_example.bin --format1 bin --file2 samples/records_example.txt --
format2 txt
```

**./target/debug/ypbank_converter** -- утилита для преобразования формата файла. Параметры могут передваться в различном порядке, лишь бы они были все указаны

* `--input` -- абсолютный/относительный путь до файла исходного файла
* `--in-format` -- формат исходного файла, значения: csv, txt, bin
* `--output-format` -- выходной формат данных, значения: csv, txt, bin

Пример запуска

```
./target/debug/ypbank_converter --input samples/records_example.txt --input-format txt --output-format txt
```