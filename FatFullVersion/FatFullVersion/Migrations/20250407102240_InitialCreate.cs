using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace FatFullVersion.Migrations
{
    /// <inheritdoc />
    public partial class InitialCreate : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.CreateTable(
                name: "ComparisonTables",
                columns: table => new
                {
                    Id = table.Column<int>(type: "INTEGER", nullable: false)
                        .Annotation("Sqlite:Autoincrement", true),
                    ChannelAddress = table.Column<string>(type: "TEXT", maxLength: 50, nullable: false),
                    CommunicationAddress = table.Column<string>(type: "TEXT", maxLength: 50, nullable: false),
                    ChannelType = table.Column<int>(type: "INTEGER", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_ComparisonTables", x => x.Id);
                });

            migrationBuilder.CreateTable(
                name: "PlcConnectionConfigs",
                columns: table => new
                {
                    Id = table.Column<int>(type: "INTEGER", nullable: false)
                        .Annotation("Sqlite:Autoincrement", true),
                    IpAddress = table.Column<string>(type: "TEXT", maxLength: 50, nullable: false),
                    Port = table.Column<int>(type: "INTEGER", nullable: false),
                    Station = table.Column<byte>(type: "INTEGER", nullable: false),
                    AddressStartWithZero = table.Column<bool>(type: "INTEGER", nullable: false),
                    IsCheckMessageId = table.Column<bool>(type: "INTEGER", nullable: false),
                    IsStringReverse = table.Column<bool>(type: "INTEGER", nullable: false),
                    DataFormat = table.Column<string>(type: "TEXT", maxLength: 20, nullable: false),
                    ConnectTimeOut = table.Column<int>(type: "INTEGER", nullable: false),
                    ReceiveTimeOut = table.Column<int>(type: "INTEGER", nullable: false),
                    SleepTime = table.Column<int>(type: "INTEGER", nullable: false),
                    SocketKeepAliveTime = table.Column<int>(type: "INTEGER", nullable: false),
                    IsTestPlc = table.Column<bool>(type: "INTEGER", nullable: false),
                    IsPersistentConnection = table.Column<bool>(type: "INTEGER", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_PlcConnectionConfigs", x => x.Id);
                });

            migrationBuilder.CreateIndex(
                name: "IX_ComparisonTables_ChannelAddress_ChannelType",
                table: "ComparisonTables",
                columns: new[] { "ChannelAddress", "ChannelType" });

            migrationBuilder.CreateIndex(
                name: "IX_PlcConnectionConfigs_IsTestPlc",
                table: "PlcConnectionConfigs",
                column: "IsTestPlc");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "ComparisonTables");

            migrationBuilder.DropTable(
                name: "PlcConnectionConfigs");
        }
    }
}
