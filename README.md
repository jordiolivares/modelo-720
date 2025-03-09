# Modelo 720 tools
This project contains a CLI utility that generates the necessary Modelo 720 files that the Spanish Tax Agency (Hacienda) requires every year containing the portfolio of assets contained outside of Spain.
This was created mostly as a necessity due to Minots moving to use ISIN codes for their Notes, which resulted in entirely too many entries to handle manually.

## Supported brokers
Right now the tool only supports two brokers, each defining their input set of files:
* Mintos (Only Loans)
* Interactive Brokers (Only ETFs)

### Interactive Brokers
Interactive Brokers currently requires a CSV file generated with the following fields:
* Description (Name of the asset)
* ISIN
* Quantity
* PositionValue

*NOTE THAT THE VALUES ARE EXPECTED TO BE IN EUROS*

### Mintos
Mintos is a bit of a special case as they technically perform investment operations 24/7. They also are not helpful in that the Modelo 720 requires the ISIN of each note and their Fiscal statement doesn't include it.

As a result of all this the Mintos parser accepts two sets of files:
* A CSV file of the Notes portion of the loans obtained via the User page at *Current Investments* > *Download Selected list*
* The file above plus the account statement of operations to revert

This is necessary because the first one provides a snapshot of the portfolio at the time of download, but the second provides all the operations that have been performed in a given time-range.
Most of the time you'll want to use the second option as I don't expect anyone to get online immediately at the start of the year in order to enter Mintos and download the statement.

The algorithm used to revert operations is as simple as performing the inverse of each operation on a portfolio.
* Investment into a Note -> Subtract the value from the note
* Principal Received -> Add back the principal to the note
* Repurchase of loan principal -> Reinstate the note with the given principal value left
Interest income is ignored since Modelo 720 only wants to know the principal of a debt instrument left. We also ignore Claims since they are not subject to Modelo 720.

To properly use the second approach it is thus imperative that the snapshot is stable. An algorithm to do this is to perform the following:
* Download the Account Statement
* Download the Portfolio
* Download the Account Statement again, if it matches the one from before we've acquired a proper snapshot. Otherwise repeat the process since we missed some operations.

For the purpose of Modelo 720 you'll want to get the statement for the operations performed after the 31st of December until the current date. This way you'll get a proper snapshot of what the portfolio was at the end of the year.
