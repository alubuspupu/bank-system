# streaming quotes project

Сборка проекта осуществляется с использованием команды 

```
cargo build
```

## Сервер 

Серверу необходимо указать три аргумента в строго заданной последовательности:

* адрес слушащего сокета;
* порт слушащего сокета;
* файл, содержащий начальные значения доступных кодировок для генерации в формате json.

Пример содержания файла с начальными значениями:

```
[
  { "ticker": "AAPL", "current_price": 175.5, "current_volume": 55000000 },
  { "ticker": "MSFT", "current_price": 420.2, "current_volume": 20000000 },
  { "ticker": "GOOGL", "current_price": 155.1, "current_volume": 25000000 }
]
```

Где:

* `ticker` - имя котировки;
* `curent_price` - начальное значение цены;
* `current_value` - начальное значение объема.

Пример запуска бинарника:

```
./target/debug/quote_server 127.0.0.1 12345 /home/finik/bank-system/streaming_quotes_project/bin/tickers.json
```

## Клиент 

Клиенту необходимо указать три аргумента, они могут быть в разном порядке:

* `--udp-port` - тот самый порт, который будет использоваться для слушащего сокета;
* `--server-addr` - адрес и порт сервера, например 127.0.0.1:12345";
* `--tickers-file` - файл с кодировками, информацию о которых нужно получать.

Пример запуска:

```
./target/debug/quote_client --server-addr 127.0.0.1:12345 --udp-port 12346 --tickers-file ./test
```

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