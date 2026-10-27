? Standard library tour: calendar arithmetic with the date module
import date

spell main() {
    bind launch = date.make_date(2026, 10, 27)
    cast date.format(launch, "%A, %d %B %Y")
    cast "day " + to_text(date.day_of_year(launch)) + " of the year"

    bind deadline = date.add_days(launch, 45)
    cast "deadline: " + date.iso(deadline) + " (" + date.weekday_name(deadline) + ")"
    cast "days left: " + to_text(date.diff_days(deadline, launch))

    bind month_end = date.add_months(date.make_date(2026, 1, 31), 1)
    cast "Jan 31 + 1 month = " + date.iso(month_end)

    bind meeting = date.parse("2026-11-03 09:30")
    cast date.format(meeting, "%a %b %e, %I:%M %p")
    cast date.iso(date.add_hours(meeting, 36))

    cast date.compare(launch, deadline)
    cast date.is_leap_year(2028)
    cast date.iso(date.from_timestamp(0))
}
