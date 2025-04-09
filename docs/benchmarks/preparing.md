Install local Net-SNMPd:

```
./tools/build/setup-snmpd.sh
```

Install system packages:

=== "RHEL/CentOS"

    ```
    sudo yum install net-snmp-devel
    ```

=== "Debian/Ubuntu"

    ```
    sudo apt-get install libsnmp-dev
    ```

Install dependencies:
```
pip install -r .requirements/test.txt -r .requirements/bench.txt gufo-snmp
```
