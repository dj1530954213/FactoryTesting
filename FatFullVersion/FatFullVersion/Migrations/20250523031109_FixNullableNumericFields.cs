using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace FatFullVersion.Migrations
{
    /// <inheritdoc />
    public partial class FixNullableNumericFields : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AlterColumn<float>(
                name: "Value75Percent",
                table: "ChannelMappings",
                type: "REAL",
                nullable: true,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AlterColumn<float>(
                name: "Value50Percent",
                table: "ChannelMappings",
                type: "REAL",
                nullable: true,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AlterColumn<float>(
                name: "Value25Percent",
                table: "ChannelMappings",
                type: "REAL",
                nullable: true,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AlterColumn<float>(
                name: "Value100Percent",
                table: "ChannelMappings",
                type: "REAL",
                nullable: true,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AlterColumn<float>(
                name: "Value0Percent",
                table: "ChannelMappings",
                type: "REAL",
                nullable: true,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AlterColumn<DateTime>(
                name: "StartTime",
                table: "ChannelMappings",
                type: "TEXT",
                nullable: true,
                oldClrType: typeof(DateTime),
                oldType: "TEXT");

            migrationBuilder.AlterColumn<float>(
                name: "SLSetValueNumber",
                table: "ChannelMappings",
                type: "REAL",
                nullable: true,
                oldClrType: typeof(float),
                oldType: "REAL");

            migrationBuilder.AlterColumn<float>(
                name: "SLLSetValueNumber",
                table: "ChannelMappings",
                type: "REAL",
                nullable: true,
                oldClrType: typeof(float),
                oldType: "REAL");

            migrationBuilder.AlterColumn<float>(
                name: "SHSetValueNumber",
                table: "ChannelMappings",
                type: "REAL",
                nullable: true,
                oldClrType: typeof(float),
                oldType: "REAL");

            migrationBuilder.AlterColumn<float>(
                name: "SHHSetValueNumber",
                table: "ChannelMappings",
                type: "REAL",
                nullable: true,
                oldClrType: typeof(float),
                oldType: "REAL");

            migrationBuilder.AlterColumn<float>(
                name: "RangeUpperLimitValue",
                table: "ChannelMappings",
                type: "REAL",
                nullable: true,
                oldClrType: typeof(float),
                oldType: "REAL");

            migrationBuilder.AlterColumn<float>(
                name: "RangeLowerLimitValue",
                table: "ChannelMappings",
                type: "REAL",
                nullable: true,
                oldClrType: typeof(float),
                oldType: "REAL");

            migrationBuilder.AlterColumn<float>(
                name: "ExpectedValue",
                table: "ChannelMappings",
                type: "REAL",
                nullable: true,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AlterColumn<DateTime>(
                name: "EndTime",
                table: "ChannelMappings",
                type: "TEXT",
                nullable: true,
                oldClrType: typeof(DateTime),
                oldType: "TEXT");

            migrationBuilder.AlterColumn<float>(
                name: "ActualValue",
                table: "ChannelMappings",
                type: "REAL",
                nullable: true,
                oldClrType: typeof(double),
                oldType: "REAL");

            migrationBuilder.AddColumn<int>(
                name: "AlarmValueSetStatusEnum",
                table: "ChannelMappings",
                type: "INTEGER",
                nullable: false,
                defaultValue: 0);

            migrationBuilder.AddColumn<string>(
                name: "HardPointErrorDetail",
                table: "ChannelMappings",
                type: "TEXT",
                nullable: true);

            migrationBuilder.AddColumn<int>(
                name: "HardPointStatus",
                table: "ChannelMappings",
                type: "INTEGER",
                nullable: false,
                defaultValue: 0);

            migrationBuilder.AddColumn<int>(
                name: "HighAlarmStatusEnum",
                table: "ChannelMappings",
                type: "INTEGER",
                nullable: false,
                defaultValue: 0);

            migrationBuilder.AddColumn<int>(
                name: "HighHighAlarmStatusEnum",
                table: "ChannelMappings",
                type: "INTEGER",
                nullable: false,
                defaultValue: 0);

            migrationBuilder.AddColumn<int>(
                name: "LowAlarmStatusEnum",
                table: "ChannelMappings",
                type: "INTEGER",
                nullable: false,
                defaultValue: 0);

            migrationBuilder.AddColumn<int>(
                name: "LowLowAlarmStatusEnum",
                table: "ChannelMappings",
                type: "INTEGER",
                nullable: false,
                defaultValue: 0);

            migrationBuilder.AddColumn<int>(
                name: "MaintenanceFunctionEnum",
                table: "ChannelMappings",
                type: "INTEGER",
                nullable: false,
                defaultValue: 0);

            migrationBuilder.AddColumn<int>(
                name: "OverallStatus",
                table: "ChannelMappings",
                type: "INTEGER",
                nullable: false,
                defaultValue: 0);

            migrationBuilder.AddColumn<int>(
                name: "ReportCheckEnum",
                table: "ChannelMappings",
                type: "INTEGER",
                nullable: false,
                defaultValue: 0);

            migrationBuilder.AddColumn<int>(
                name: "ShowValueStatusEnum",
                table: "ChannelMappings",
                type: "INTEGER",
                nullable: false,
                defaultValue: 0);

            migrationBuilder.AddColumn<int>(
                name: "TrendCheckEnum",
                table: "ChannelMappings",
                type: "INTEGER",
                nullable: false,
                defaultValue: 0);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "AlarmValueSetStatusEnum",
                table: "ChannelMappings");

            migrationBuilder.DropColumn(
                name: "HardPointErrorDetail",
                table: "ChannelMappings");

            migrationBuilder.DropColumn(
                name: "HardPointStatus",
                table: "ChannelMappings");

            migrationBuilder.DropColumn(
                name: "HighAlarmStatusEnum",
                table: "ChannelMappings");

            migrationBuilder.DropColumn(
                name: "HighHighAlarmStatusEnum",
                table: "ChannelMappings");

            migrationBuilder.DropColumn(
                name: "LowAlarmStatusEnum",
                table: "ChannelMappings");

            migrationBuilder.DropColumn(
                name: "LowLowAlarmStatusEnum",
                table: "ChannelMappings");

            migrationBuilder.DropColumn(
                name: "MaintenanceFunctionEnum",
                table: "ChannelMappings");

            migrationBuilder.DropColumn(
                name: "OverallStatus",
                table: "ChannelMappings");

            migrationBuilder.DropColumn(
                name: "ReportCheckEnum",
                table: "ChannelMappings");

            migrationBuilder.DropColumn(
                name: "ShowValueStatusEnum",
                table: "ChannelMappings");

            migrationBuilder.DropColumn(
                name: "TrendCheckEnum",
                table: "ChannelMappings");

            migrationBuilder.AlterColumn<double>(
                name: "Value75Percent",
                table: "ChannelMappings",
                type: "REAL",
                nullable: false,
                defaultValue: 0.0,
                oldClrType: typeof(float),
                oldType: "REAL",
                oldNullable: true);

            migrationBuilder.AlterColumn<double>(
                name: "Value50Percent",
                table: "ChannelMappings",
                type: "REAL",
                nullable: false,
                defaultValue: 0.0,
                oldClrType: typeof(float),
                oldType: "REAL",
                oldNullable: true);

            migrationBuilder.AlterColumn<double>(
                name: "Value25Percent",
                table: "ChannelMappings",
                type: "REAL",
                nullable: false,
                defaultValue: 0.0,
                oldClrType: typeof(float),
                oldType: "REAL",
                oldNullable: true);

            migrationBuilder.AlterColumn<double>(
                name: "Value100Percent",
                table: "ChannelMappings",
                type: "REAL",
                nullable: false,
                defaultValue: 0.0,
                oldClrType: typeof(float),
                oldType: "REAL",
                oldNullable: true);

            migrationBuilder.AlterColumn<double>(
                name: "Value0Percent",
                table: "ChannelMappings",
                type: "REAL",
                nullable: false,
                defaultValue: 0.0,
                oldClrType: typeof(float),
                oldType: "REAL",
                oldNullable: true);

            migrationBuilder.AlterColumn<DateTime>(
                name: "StartTime",
                table: "ChannelMappings",
                type: "TEXT",
                nullable: false,
                defaultValue: new DateTime(1, 1, 1, 0, 0, 0, 0, DateTimeKind.Unspecified),
                oldClrType: typeof(DateTime),
                oldType: "TEXT",
                oldNullable: true);

            migrationBuilder.AlterColumn<float>(
                name: "SLSetValueNumber",
                table: "ChannelMappings",
                type: "REAL",
                nullable: false,
                defaultValue: 0f,
                oldClrType: typeof(float),
                oldType: "REAL",
                oldNullable: true);

            migrationBuilder.AlterColumn<float>(
                name: "SLLSetValueNumber",
                table: "ChannelMappings",
                type: "REAL",
                nullable: false,
                defaultValue: 0f,
                oldClrType: typeof(float),
                oldType: "REAL",
                oldNullable: true);

            migrationBuilder.AlterColumn<float>(
                name: "SHSetValueNumber",
                table: "ChannelMappings",
                type: "REAL",
                nullable: false,
                defaultValue: 0f,
                oldClrType: typeof(float),
                oldType: "REAL",
                oldNullable: true);

            migrationBuilder.AlterColumn<float>(
                name: "SHHSetValueNumber",
                table: "ChannelMappings",
                type: "REAL",
                nullable: false,
                defaultValue: 0f,
                oldClrType: typeof(float),
                oldType: "REAL",
                oldNullable: true);

            migrationBuilder.AlterColumn<float>(
                name: "RangeUpperLimitValue",
                table: "ChannelMappings",
                type: "REAL",
                nullable: false,
                defaultValue: 0f,
                oldClrType: typeof(float),
                oldType: "REAL",
                oldNullable: true);

            migrationBuilder.AlterColumn<float>(
                name: "RangeLowerLimitValue",
                table: "ChannelMappings",
                type: "REAL",
                nullable: false,
                defaultValue: 0f,
                oldClrType: typeof(float),
                oldType: "REAL",
                oldNullable: true);

            migrationBuilder.AlterColumn<double>(
                name: "ExpectedValue",
                table: "ChannelMappings",
                type: "REAL",
                nullable: false,
                defaultValue: 0.0,
                oldClrType: typeof(float),
                oldType: "REAL",
                oldNullable: true);

            migrationBuilder.AlterColumn<DateTime>(
                name: "EndTime",
                table: "ChannelMappings",
                type: "TEXT",
                nullable: false,
                defaultValue: new DateTime(1, 1, 1, 0, 0, 0, 0, DateTimeKind.Unspecified),
                oldClrType: typeof(DateTime),
                oldType: "TEXT",
                oldNullable: true);

            migrationBuilder.AlterColumn<double>(
                name: "ActualValue",
                table: "ChannelMappings",
                type: "REAL",
                nullable: false,
                defaultValue: 0.0,
                oldClrType: typeof(float),
                oldType: "REAL",
                oldNullable: true);
        }
    }
}
