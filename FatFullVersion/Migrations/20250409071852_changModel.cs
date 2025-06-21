using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace FatFullVersion.Migrations
{
    /// <inheritdoc />
    public partial class changModel : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropIndex(
                name: "IX_ComparisonTables_ChannelAddress_ChannelType",
                table: "ComparisonTables");

            migrationBuilder.DropPrimaryKey(
                name: "PK_PlcConnectionConfigs",
                table: "PlcConnectionConfigs");

            migrationBuilder.DropIndex(
                name: "IX_PlcConnectionConfigs_IsTestPlc",
                table: "PlcConnectionConfigs");

            migrationBuilder.RenameTable(
                name: "PlcConnectionConfigs",
                newName: "PlcConnections");

            migrationBuilder.AlterColumn<string>(
                name: "CommunicationAddress",
                table: "ComparisonTables",
                type: "TEXT",
                nullable: true,
                oldClrType: typeof(string),
                oldType: "TEXT",
                oldMaxLength: 50);

            migrationBuilder.AlterColumn<string>(
                name: "ChannelAddress",
                table: "ComparisonTables",
                type: "TEXT",
                nullable: true,
                oldClrType: typeof(string),
                oldType: "TEXT",
                oldMaxLength: 50);

            migrationBuilder.AlterColumn<string>(
                name: "IpAddress",
                table: "PlcConnections",
                type: "TEXT",
                nullable: true,
                oldClrType: typeof(string),
                oldType: "TEXT",
                oldMaxLength: 50);

            migrationBuilder.AlterColumn<string>(
                name: "DataFormat",
                table: "PlcConnections",
                type: "TEXT",
                nullable: true,
                oldClrType: typeof(string),
                oldType: "TEXT",
                oldMaxLength: 20);

            migrationBuilder.AddPrimaryKey(
                name: "PK_PlcConnections",
                table: "PlcConnections",
                column: "Id");

            migrationBuilder.CreateTable(
                name: "ChannelMappings",
                columns: table => new
                {
                    Id = table.Column<Guid>(type: "TEXT", nullable: false),
                    TestTag = table.Column<string>(type: "TEXT", nullable: true),
                    ModuleName = table.Column<string>(type: "TEXT", nullable: true),
                    ModuleType = table.Column<string>(type: "TEXT", nullable: true),
                    PowerSupplyType = table.Column<string>(type: "TEXT", nullable: true),
                    WireSystem = table.Column<string>(type: "TEXT", nullable: true),
                    Tag = table.Column<string>(type: "TEXT", nullable: true),
                    StationName = table.Column<string>(type: "TEXT", nullable: true),
                    VariableName = table.Column<string>(type: "TEXT", nullable: true),
                    VariableDescription = table.Column<string>(type: "TEXT", nullable: true),
                    DataType = table.Column<string>(type: "TEXT", nullable: true),
                    ChannelTag = table.Column<string>(type: "TEXT", nullable: true),
                    AccessProperty = table.Column<string>(type: "TEXT", nullable: true),
                    SaveHistory = table.Column<string>(type: "TEXT", nullable: true),
                    PowerFailureProtection = table.Column<string>(type: "TEXT", nullable: true),
                    RangeLowerLimit = table.Column<string>(type: "TEXT", nullable: true),
                    RangeLowerLimitValue = table.Column<float>(type: "REAL", nullable: false),
                    RangeUpperLimit = table.Column<string>(type: "TEXT", nullable: true),
                    RangeUpperLimitValue = table.Column<float>(type: "REAL", nullable: false),
                    SLLSetValue = table.Column<string>(type: "TEXT", nullable: true),
                    SLLSetValueNumber = table.Column<float>(type: "REAL", nullable: false),
                    SLLSetPoint = table.Column<string>(type: "TEXT", nullable: true),
                    SLLSetPointPLCAddress = table.Column<string>(type: "TEXT", nullable: true),
                    SLLSetPointCommAddress = table.Column<string>(type: "TEXT", nullable: true),
                    SLSetValue = table.Column<string>(type: "TEXT", nullable: true),
                    SLSetValueNumber = table.Column<float>(type: "REAL", nullable: false),
                    SLSetPoint = table.Column<string>(type: "TEXT", nullable: true),
                    SLSetPointPLCAddress = table.Column<string>(type: "TEXT", nullable: true),
                    SLSetPointCommAddress = table.Column<string>(type: "TEXT", nullable: true),
                    SHSetValue = table.Column<string>(type: "TEXT", nullable: true),
                    SHSetValueNumber = table.Column<float>(type: "REAL", nullable: false),
                    SHSetPoint = table.Column<string>(type: "TEXT", nullable: true),
                    SHSetPointPLCAddress = table.Column<string>(type: "TEXT", nullable: true),
                    SHSetPointCommAddress = table.Column<string>(type: "TEXT", nullable: true),
                    SHHSetValue = table.Column<string>(type: "TEXT", nullable: true),
                    SHHSetValueNumber = table.Column<float>(type: "REAL", nullable: false),
                    SHHSetPoint = table.Column<string>(type: "TEXT", nullable: true),
                    SHHSetPointPLCAddress = table.Column<string>(type: "TEXT", nullable: true),
                    SHHSetPointCommAddress = table.Column<string>(type: "TEXT", nullable: true),
                    LLAlarm = table.Column<string>(type: "TEXT", nullable: true),
                    LLAlarmPLCAddress = table.Column<string>(type: "TEXT", nullable: true),
                    LLAlarmCommAddress = table.Column<string>(type: "TEXT", nullable: true),
                    LAlarm = table.Column<string>(type: "TEXT", nullable: true),
                    LAlarmPLCAddress = table.Column<string>(type: "TEXT", nullable: true),
                    LAlarmCommAddress = table.Column<string>(type: "TEXT", nullable: true),
                    HAlarm = table.Column<string>(type: "TEXT", nullable: true),
                    HAlarmPLCAddress = table.Column<string>(type: "TEXT", nullable: true),
                    HAlarmCommAddress = table.Column<string>(type: "TEXT", nullable: true),
                    HHAlarm = table.Column<string>(type: "TEXT", nullable: true),
                    HHAlarmPLCAddress = table.Column<string>(type: "TEXT", nullable: true),
                    HHAlarmCommAddress = table.Column<string>(type: "TEXT", nullable: true),
                    MaintenanceValueSetting = table.Column<string>(type: "TEXT", nullable: true),
                    MaintenanceValueSetPoint = table.Column<string>(type: "TEXT", nullable: true),
                    MaintenanceValueSetPointPLCAddress = table.Column<string>(type: "TEXT", nullable: true),
                    MaintenanceValueSetPointCommAddress = table.Column<string>(type: "TEXT", nullable: true),
                    MaintenanceEnableSwitchPoint = table.Column<string>(type: "TEXT", nullable: true),
                    MaintenanceEnableSwitchPointPLCAddress = table.Column<string>(type: "TEXT", nullable: true),
                    MaintenanceEnableSwitchPointCommAddress = table.Column<string>(type: "TEXT", nullable: true),
                    PLCAbsoluteAddress = table.Column<string>(type: "TEXT", nullable: true),
                    PlcCommunicationAddress = table.Column<string>(type: "TEXT", nullable: true),
                    CreatedTime = table.Column<DateTime>(type: "TEXT", nullable: false, defaultValueSql: "CURRENT_TIMESTAMP"),
                    UpdatedTime = table.Column<DateTime>(type: "TEXT", nullable: true),
                    TestBatch = table.Column<string>(type: "TEXT", nullable: true),
                    TestPLCChannelTag = table.Column<string>(type: "TEXT", nullable: true),
                    TestPLCCommunicationAddress = table.Column<string>(type: "TEXT", nullable: true),
                    MonitorStatus = table.Column<string>(type: "TEXT", nullable: true),
                    TestId = table.Column<int>(type: "INTEGER", nullable: false),
                    TestResultStatus = table.Column<int>(type: "INTEGER", nullable: false),
                    ResultText = table.Column<string>(type: "TEXT", nullable: true),
                    HardPointTestResult = table.Column<string>(type: "TEXT", nullable: true),
                    TestTime = table.Column<DateTime>(type: "TEXT", nullable: true),
                    FinalTestTime = table.Column<DateTime>(type: "TEXT", nullable: true),
                    Status = table.Column<string>(type: "TEXT", nullable: true),
                    StartTime = table.Column<DateTime>(type: "TEXT", nullable: false),
                    EndTime = table.Column<DateTime>(type: "TEXT", nullable: false),
                    ExpectedValue = table.Column<double>(type: "REAL", nullable: false),
                    ActualValue = table.Column<double>(type: "REAL", nullable: false),
                    Value0Percent = table.Column<double>(type: "REAL", nullable: false),
                    Value25Percent = table.Column<double>(type: "REAL", nullable: false),
                    Value50Percent = table.Column<double>(type: "REAL", nullable: false),
                    Value75Percent = table.Column<double>(type: "REAL", nullable: false),
                    Value100Percent = table.Column<double>(type: "REAL", nullable: false),
                    LowLowAlarmStatus = table.Column<string>(type: "TEXT", nullable: true),
                    LowAlarmStatus = table.Column<string>(type: "TEXT", nullable: true),
                    HighAlarmStatus = table.Column<string>(type: "TEXT", nullable: true),
                    HighHighAlarmStatus = table.Column<string>(type: "TEXT", nullable: true),
                    MaintenanceFunction = table.Column<string>(type: "TEXT", nullable: true),
                    ErrorMessage = table.Column<string>(type: "TEXT", nullable: true),
                    CurrentValue = table.Column<string>(type: "TEXT", nullable: true),
                    ShowValueStatus = table.Column<string>(type: "TEXT", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_ChannelMappings", x => x.Id);
                });

            migrationBuilder.CreateIndex(
                name: "IX_ChannelMappings_TestTag",
                table: "ChannelMappings",
                column: "TestTag");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "ChannelMappings");

            migrationBuilder.DropPrimaryKey(
                name: "PK_PlcConnections",
                table: "PlcConnections");

            migrationBuilder.RenameTable(
                name: "PlcConnections",
                newName: "PlcConnectionConfigs");

            migrationBuilder.AlterColumn<string>(
                name: "CommunicationAddress",
                table: "ComparisonTables",
                type: "TEXT",
                maxLength: 50,
                nullable: false,
                defaultValue: "",
                oldClrType: typeof(string),
                oldType: "TEXT",
                oldNullable: true);

            migrationBuilder.AlterColumn<string>(
                name: "ChannelAddress",
                table: "ComparisonTables",
                type: "TEXT",
                maxLength: 50,
                nullable: false,
                defaultValue: "",
                oldClrType: typeof(string),
                oldType: "TEXT",
                oldNullable: true);

            migrationBuilder.AlterColumn<string>(
                name: "IpAddress",
                table: "PlcConnectionConfigs",
                type: "TEXT",
                maxLength: 50,
                nullable: false,
                defaultValue: "",
                oldClrType: typeof(string),
                oldType: "TEXT",
                oldNullable: true);

            migrationBuilder.AlterColumn<string>(
                name: "DataFormat",
                table: "PlcConnectionConfigs",
                type: "TEXT",
                maxLength: 20,
                nullable: false,
                defaultValue: "",
                oldClrType: typeof(string),
                oldType: "TEXT",
                oldNullable: true);

            migrationBuilder.AddPrimaryKey(
                name: "PK_PlcConnectionConfigs",
                table: "PlcConnectionConfigs",
                column: "Id");

            migrationBuilder.CreateIndex(
                name: "IX_ComparisonTables_ChannelAddress_ChannelType",
                table: "ComparisonTables",
                columns: new[] { "ChannelAddress", "ChannelType" });

            migrationBuilder.CreateIndex(
                name: "IX_PlcConnectionConfigs_IsTestPlc",
                table: "PlcConnectionConfigs",
                column: "IsTestPlc");
        }
    }
}
