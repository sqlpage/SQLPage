Name:           sqlpage
Version:        0.38.0
Release:        0.1.beta.1%{?dist}
Summary:        SQL-only webapp builder

License:        MIT
URL:            https://sql-page.com
Source0:        https://github.com/sqlpage/SQLPage/archive/v%{version}.tar.gz

BuildRequires:  rust >= 1.70
BuildRequires:  cargo >= 1.70
BuildRequires:  openssl-devel
BuildRequires:  systemd-rpm-macros
BuildRequires:  unixODBC-devel
BuildRequires:  freetds-devel

Requires:       unixODBC
Recommends:     sqlite
Recommends:     postgresql
Recommends:     mariadb

%{?systemd_requires}

%description
SQLPage is a web server that takes .sql files and formats the query
results using pre-made configurable professional-looking components.

With SQLPage, you write simple .sql files containing queries to your
database to select, group, update, insert, and delete your data, and
you get good-looking clean webpages displaying your data as text,
lists, grids, plots, and forms.

Supported databases include SQLite, PostgreSQL, MySQL, Microsoft SQL
Server, and any ODBC-compatible database such as ClickHouse, MongoDB,
DuckDB, Oracle, Snowflake, BigQuery, and IBM DB2.

%prep
%setup -q -n SQLPage-%{version}

%build
export CARGO_HOME=$(pwd)/.cargo
cargo build --profile superoptimized --locked --release

%install
rm -rf %{buildroot}

# Install binary
install -D -m 755 target/superoptimized/sqlpage %{buildroot}%{_bindir}/sqlpage

# Install systemd service
install -D -m 644 sqlpage.service %{buildroot}%{_unitdir}/sqlpage.service

# Install configuration and data files
install -d %{buildroot}%{_sysconfdir}/sqlpage
install -d %{buildroot}%{_sharedstatedir}/sqlpage
install -d %{buildroot}/var/www/sqlpage

install -D -m 644 sqlpage/sqlpage.json %{buildroot}%{_sysconfdir}/sqlpage/sqlpage.json
cp -r sqlpage/templates %{buildroot}%{_sysconfdir}/sqlpage/
cp -r sqlpage/migrations %{buildroot}%{_sysconfdir}/sqlpage/

install -D -m 644 sqlpage/favicon.svg %{buildroot}%{_sysconfdir}/sqlpage/favicon.svg
install -D -m 644 sqlpage/tabler-icons.svg %{buildroot}%{_sysconfdir}/sqlpage/tabler-icons.svg
install -D -m 644 sqlpage/apexcharts.js %{buildroot}%{_sysconfdir}/sqlpage/apexcharts.js
install -D -m 644 sqlpage/tomselect.js %{buildroot}%{_sysconfdir}/sqlpage/tomselect.js
install -D -m 644 sqlpage/sqlpage.css %{buildroot}%{_sysconfdir}/sqlpage/sqlpage.css
install -D -m 644 sqlpage/sqlpage.js %{buildroot}%{_sysconfdir}/sqlpage/sqlpage.js

%pre
getent group sqlpage >/dev/null || groupadd -r sqlpage
getent passwd sqlpage >/dev/null || \
    useradd -r -g sqlpage -d /var/www/sqlpage -s /sbin/nologin \
    -c "SQLPage web server" sqlpage
exit 0

%post
%systemd_post sqlpage.service

# Create log directory
mkdir -p /var/log/sqlpage
chown sqlpage:sqlpage /var/log/sqlpage
chmod 750 /var/log/sqlpage

# Set ownership
chown -R sqlpage:sqlpage /var/www/sqlpage
chmod 755 /var/www/sqlpage

%preun
%systemd_preun sqlpage.service

%postun
%systemd_postun_with_restart sqlpage.service

if [ $1 -eq 0 ]; then
    # Package removal, not upgrade
    userdel sqlpage 2>/dev/null || :
    groupdel sqlpage 2>/dev/null || :
    rm -rf /var/log/sqlpage
fi

%files
%license LICENSE.txt
%doc README.md CHANGELOG.md
%{_bindir}/sqlpage
%{_unitdir}/sqlpage.service
%dir %{_sysconfdir}/sqlpage
%config(noreplace) %{_sysconfdir}/sqlpage/sqlpage.json
%{_sysconfdir}/sqlpage/templates/
%{_sysconfdir}/sqlpage/migrations/
%{_sysconfdir}/sqlpage/*.svg
%{_sysconfdir}/sqlpage/*.js
%{_sysconfdir}/sqlpage/*.css
%dir %attr(755,sqlpage,sqlpage) /var/www/sqlpage
%dir %attr(750,sqlpage,sqlpage) /var/log/sqlpage

%changelog
* Thu Oct 02 2025 SQLPage Contributors <sqlpage@sql-page.com> - 0.38.0-0.1.beta.1
- Initial RPM package release
- SQL-only webapp builder with support for multiple databases
- Includes systemd service configuration
- Support for SQLite, PostgreSQL, MySQL, MS SQL Server, and ODBC
