using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace FatFullVersion.Migrations
{
    /// <inheritdoc />
    public partial class addPowerSupplyType : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<string>(
                name: "PowerSupplyType",
                table: "ComparisonTables",
                type: "TEXT",
                nullable: true);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "PowerSupplyType",
                table: "ComparisonTables");
        }
    }
}
