---
icon: timeline
category: display-data
summary: Plot query results as line, area, bar, or pie charts.
top:
  - class
  - name: color
    type: COLOR
    description: The name of a color in which to display the chart. If there are multiple series in the chart,
      this parameter can be repeated multiple times.
  - name: height
    type: INTEGER
    description: 'Height of the chart, in pixels. By default: 250'
  - name: horizontal
    type: BOOLEAN
    description: Displays a bar chart with horizontal bars instead of vertical ones.
  - id
  - name: labels
    type: BOOLEAN
    description: Whether to show the data labels on the chart or not.
  - name: logarithmic
    type: BOOLEAN
    description: Display the y-axis in logarithmic scale.
  - name: marker
    type: REAL
    description: Marker size
  - name: show_legend
    type: BOOLEAN
    description: Whether to display the legend listing all chart series. Defaults to true.
  - name: stacked
    type: BOOLEAN
    description: Whether to cumulate values from different series. Supported by the "line", "area" and "bar"
      chart types, and ignored by the others. Series are aligned on their x values, and a series that has no
      value for a given x counts as zero there.
  - name: time
    type: BOOLEAN
    description: Whether the x-axis represents time. If set to true, the x values will be parsed and formatted
      as dates for the user.
  - name: title
    type: TEXT
    description: The name of the chart.
  - name: toolbar
    type: BOOLEAN
    description: Whether to display a toolbar at the top right of the chart, that offers downloading the data
      as CSV.
  - name: type
    type: TEXT
    required: true
    description: 'The type of chart. One of: "line", "area", "bar", "column", "pie", "scatter", "bubble", "heatmap",
      "rangeBar". "column" is a synonym of "bar".'
  - name: xmax
    type: TEXT
    description: The maximum value for the x-axis. When time is true, this can be a date or timestamp.
  - name: xmin
    type: TEXT
    description: The minimal value for the x-axis. When time is true, this can be a date or timestamp.
  - name: xticks
    type: INTEGER
    description: Number of intervals used to generate a numeric x-axis, normally producing one more tick position.
      On category and time axes, this is a target for label density, so the visible label count may differ.
  - name: xtitle
    type: TEXT
    description: Title of the x axis, displayed below it.
  - name: ymax
    type: REAL
    description: The maximum value for the y-axis.
  - name: ymin
    type: REAL
    description: The minimal value for the y-axis.
  - name: ystep
    type: REAL
    description: Step between ticks on the y axis.
  - name: yticks
    type: INTEGER
    description: Number of ticks on the y axis.
  - name: ytitle
    type: TEXT
    description: Title of the y axis, displayed to its left.
  - name: ztitle
    type: TEXT
    description: Title of the z axis, displayed in tooltips.
row:
  - name: color
    type: COLOR
    description: 'The name of a color for what this row draws: the bar, slice or point it plots, or the reference
      line it draws. Defaults to the color of the series for a data point, and to grey for a reference line.'
  - name: label
    type: TEXT
    description: An alias for parameter "x". On a row that draws a reference line, the text to display next
      to the line. Labels are plain text; HTML markup is displayed literally.
  - name: link
    type: URL
    description: A URL to open when the data point is clicked. The tooltip value also links to the point, as
      does a text x value when shown as the tooltip title.
  - name: series
    type: TEXT
    description: If multiple series are represented and share the same y-axis, this parameter can be used to
      distinguish between them. Series names are plain text; HTML markup is displayed literally.
  - name: value
    type: REAL
    description: An alias for parameter "y"
  - name: x
    type: REAL
    required: true
    description: The value of the point on the horizontal axis. Numeric values use continuous, proportionate
      positioning; text values are evenly spaced categories. Set the top-level time property for dates and
      timestamps.
  - name: xline
    type: TEXT
    description: Draws a reference line across the chart at this position of the x axis instead of plotting
      a point, to mark an event such as a deployment. A date or a timestamp when time is set, otherwise one
      of the x values.
  - name: xline_end
    type: TEXT
    description: Makes the xline a band instead of a line, reaching to this value.
  - name: y
    type: REAL
    required: true
    description: The value of the point on the vertical axis
  - name: yline
    type: REAL
    description: Draws a reference line across the chart at this value of the y axis instead of plotting a
      point, to show a limit such as a quota or an alarm threshold. Not drawn if it falls outside of the axis,
      so set ymax when the limit is above the data.
  - name: yline_end
    type: REAL
    description: Makes the yline a band instead of a line, reaching to this value.
  - name: z
    type: REAL
    description: A third value carried by the point. Used as the bubble radius in a bubble chart, and shown
      in the tooltip under the name given by the top-level "ztitle".
---

A component that plots data. Line, area, bar, and pie charts are all supported. Each item in the component is a data point in the graph.
