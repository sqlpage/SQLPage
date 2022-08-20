for (const c of document.getElementsByClassName("chart")) {
  try {
    const series = {};
    JSON.parse(c.dataset.contents).forEach(([name, x, y]) => {
      series[name] = series[name] || { name, data: [] }
      series[name].data.push([x, y]);
    })
    c.innerHTML = "";
    new ApexCharts(c, {
      chart: {
        type: 'line',
        fontFamily: 'inherit',
        parentHeightOffset: 0,
        height: 250,
        toolbar: {
          show: false,
        },
        animations: {
          enabled: false
        }
      },
      dataLabels: {
        enabled: false,
      },
      fill: {
        opacity: .16,
        type: 'solid'
      },
      stroke: {
        width: 4,
        lineCap: "round",
        curve: "smooth",
      },
      xaxis: {
        labels: {
          padding: 0,
        },
        tooltip: {
          enabled: false
        },
        axisBorder: {
          show: false,
        },
      },
      series: Object.values(series),
    }).render()
  } catch (e) { console.log(e) }
}
