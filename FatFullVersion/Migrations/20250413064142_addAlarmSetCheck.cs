using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace FatFullVersion.Migrations
{
    /// <inheritdoc />
    public partial class addAlarmSetCheck : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<string>(
                name: "AlarmValueSetStatus",
                table: "ChannelMappings",
                type: "TEXT",
                nullable: true);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "AlarmValueSetStatus",
                table: "ChannelMappings");
        }
    }
}
